use crate::app::{App, AppMode, ConfirmAction, FlatItem};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

pub fn handle_event(app: &mut App) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match &app.mode {
                AppMode::Normal => handle_normal_mode(app, key),
                AppMode::Adding { .. } => handle_input_mode(app, key),
                AppMode::Editing { .. } => handle_input_mode(app, key),
                AppMode::Confirm { .. } => handle_confirm_mode(app, key),
                AppMode::Help => handle_help_mode(app, key),
            }
        }
    }
    Ok(())
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    // Clear status message on any key press
    app.status_message = None;

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.dirty {
                app.mode = AppMode::Confirm {
                    message: "You have unsaved changes. Quit anyway?".to_string(),
                    action: ConfirmAction::Quit,
                };
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('?') => {
            app.mode = AppMode::Help;
        }
        KeyCode::Tab => {
            app.next_tab();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            move_selection(app, 1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            move_selection(app, -1);
        }
        KeyCode::Char('h') | KeyCode::Left => {
            collapse_current(app);
        }
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            expand_or_select(app);
        }
        KeyCode::Char('a') => {
            app.mode = AppMode::Adding {
                input: String::new(),
            };
        }
        KeyCode::Char('e') => {
            if let Some(index) = get_selected_permission_index(app) {
                let perms = app.current_permissions();
                if index < perms.len() {
                    app.mode = AppMode::Editing {
                        index,
                        input: perms[index].clone(),
                    };
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(index) = get_selected_permission_index(app) {
                app.mode = AppMode::Confirm {
                    message: "Delete this permission?".to_string(),
                    action: ConfirmAction::Delete(index),
                };
            }
        }
        KeyCode::Char('s') => {
            if let Err(e) = app.save() {
                app.status_message = Some(format!("Error: {}", e));
            }
        }
        KeyCode::Char('r') => {
            if let Err(e) = app.reload() {
                app.status_message = Some(format!("Error: {}", e));
            }
        }
        KeyCode::Char('g') => {
            app.selected_source = crate::app::SettingsSource::Global;
            app.tree_state = crate::app::TreeState::default();
            app.status_message = Some("Global settings".to_string());
        }
        KeyCode::Char('G') => {
            // Jump to bottom
            let items = app.build_flat_items();
            if !items.is_empty() {
                app.tree_state.flat_index = items.len() - 1;
            }
        }
        KeyCode::Char('L') => {
            app.selected_source = crate::app::SettingsSource::Local;
            app.tree_state = crate::app::TreeState::default();
            app.status_message = Some("Local settings".to_string());
        }
        _ => {}
    }
}

fn handle_input_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter => {
            match &app.mode {
                AppMode::Adding { input } => {
                    if !input.trim().is_empty() {
                        app.add_permission(input.trim().to_string());
                        app.status_message = Some("Permission added".to_string());
                    }
                }
                AppMode::Editing { index, input } => {
                    if !input.trim().is_empty() {
                        app.edit_permission(*index, input.trim().to_string());
                        app.status_message = Some("Permission updated".to_string());
                    }
                }
                _ => {}
            }
            app.mode = AppMode::Normal;
        }
        KeyCode::Char(c) => {
            match &mut app.mode {
                AppMode::Adding { input } | AppMode::Editing { input, .. } => {
                    input.push(c);
                }
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match &mut app.mode {
                AppMode::Adding { input } | AppMode::Editing { input, .. } => {
                    input.pop();
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let AppMode::Confirm { action, .. } = &app.mode {
                match action {
                    ConfirmAction::Delete(index) => {
                        let idx = *index;
                        app.delete_permission(idx);
                        app.status_message = Some("Permission deleted".to_string());
                    }
                    ConfirmAction::Save => {
                        if let Err(e) = app.save() {
                            app.status_message = Some(format!("Error: {}", e));
                        }
                    }
                    ConfirmAction::Quit => {
                        app.should_quit = true;
                    }
                }
            }
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
}

fn handle_help_mode(app: &mut App, _key: KeyEvent) {
    app.mode = AppMode::Normal;
}

fn move_selection(app: &mut App, delta: i32) {
    let items = app.build_flat_items();
    if items.is_empty() {
        return;
    }

    let new_index = if delta > 0 {
        (app.tree_state.flat_index + delta as usize).min(items.len() - 1)
    } else {
        app.tree_state.flat_index.saturating_sub((-delta) as usize)
    };

    app.tree_state.flat_index = new_index;
}

fn collapse_current(app: &mut App) {
    let items = app.build_flat_items();
    if let Some(item) = items.get(app.tree_state.flat_index) {
        match item {
            FlatItem::Category { category, .. } => {
                app.toggle_category(category);
            }
            FlatItem::Permission { .. } => {
                // Find parent category and collapse it
                for (idx, item) in items.iter().enumerate().rev() {
                    if idx < app.tree_state.flat_index {
                        if let FlatItem::Category { category, .. } = item {
                            app.toggle_category(category);
                            app.tree_state.flat_index = idx;
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn expand_or_select(app: &mut App) {
    let items = app.build_flat_items();
    if let Some(item) = items.get(app.tree_state.flat_index) {
        if let FlatItem::Category { category, expanded, .. } = item {
            if !expanded {
                app.toggle_category(category);
            }
        }
    }
}

fn get_selected_permission_index(app: &App) -> Option<usize> {
    let items = app.build_flat_items();
    if let Some(FlatItem::Permission { index, .. }) = items.get(app.tree_state.flat_index) {
        Some(*index)
    } else {
        None
    }
}
