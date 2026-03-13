use crate::app::{App, AppMode, ConfirmAction, FlatItem, SettingsSource};
use crate::config::{Permission, PermissionType};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;
use tui_textarea::TextArea;

pub fn handle_event(app: &mut App) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match &app.mode {
                AppMode::Normal => handle_normal_mode(app, key),
                AppMode::Adding { .. } => handle_input_mode(app, key),
                AppMode::Editing { .. } => handle_input_mode(app, key),
                AppMode::Confirm { .. } => handle_confirm_mode(app, key),
                AppMode::Moving { .. } => handle_moving_mode(app, key),
                AppMode::Changing { .. } => handle_changing_mode(app, key),
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
            if !app.dirty.is_empty() {
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
        KeyCode::Right | KeyCode::Enter => {
            expand_or_select(app);
        }
        KeyCode::Char('a') => {
            app.textarea = Some(TextArea::default());
            app.mode = AppMode::Adding;
        }
        KeyCode::Char('e') => {
            if let Some(index) = get_selected_permission_index(app) {
                let perms = app.current_permissions();
                if index < perms.len() {
                    let mut textarea = TextArea::from([perms[index].as_str()]);
                    textarea.move_cursor(tui_textarea::CursorMove::End);
                    app.textarea = Some(textarea);
                    app.mode = AppMode::Editing { index };
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
            if app.dirty.is_empty() {
                app.status_message = Some("No unsaved changes".to_string());
            } else {
                let sources: Vec<&str> = [SettingsSource::User, SettingsSource::Project, SettingsSource::Local]
                    .iter()
                    .filter(|s| app.dirty.contains(s))
                    .map(|s| s.label())
                    .collect();
                let message = format!("Save changes to {}?", sources.join(", "));
                app.mode = AppMode::Confirm {
                    message,
                    action: ConfirmAction::Save,
                };
            }
        }
        KeyCode::Char('r') => {
            if let Err(e) = app.reload() {
                app.status_message = Some(format!("Error: {}", e));
            }
        }
        KeyCode::Char('u') => {
            app.set_source(crate::app::SettingsSource::User);
        }
        KeyCode::Char('p') => {
            app.set_source(crate::app::SettingsSource::Project);
        }
        KeyCode::Char('l') => {
            app.set_source(crate::app::SettingsSource::Local);
        }
        KeyCode::Char('o') => {
            app.sort_permissions();
            app.status_message = Some("Permissions sorted".to_string());
        }
        KeyCode::Char('m') => {
            if let Some(index) = get_selected_permission_index(app) {
                let all_sources = [SettingsSource::User, SettingsSource::Project, SettingsSource::Local];
                let destinations: Vec<SettingsSource> = all_sources
                    .iter()
                    .filter(|s| **s != app.selected_source)
                    .filter(|s| **s == SettingsSource::User || app.project_root.is_some())
                    .copied()
                    .collect();

                if destinations.is_empty() {
                    app.status_message = Some("No other settings sources available".to_string());
                } else {
                    let perm = app.current_permissions()[index].clone();
                    app.mode = AppMode::Moving {
                        index,
                        permission: perm,
                        destinations,
                        selected: 0,
                    };
                }
            }
        }
        KeyCode::Char('c') => {
            if let Some(index) = get_selected_permission_index(app) {
                let all_types = [PermissionType::Allow, PermissionType::Deny, PermissionType::Ask];
                let destinations: Vec<PermissionType> = all_types
                    .iter()
                    .filter(|t| **t != app.selected_tab)
                    .cloned()
                    .collect();

                let perm = app.current_permissions()[index].clone();
                app.mode = AppMode::Changing {
                    index,
                    permission: perm,
                    destinations,
                    selected: 0,
                };
            }
        }
        KeyCode::Char('G') => {
            let items = app.build_flat_items();
            if !items.is_empty() {
                app.tree_state.flat_index = items.len() - 1;
            }
        }
        _ => {}
    }
}

fn handle_input_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.textarea = None;
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter => {
            if let Some(ref textarea) = app.textarea {
                let input = textarea.lines()[0].trim().to_string();
                if input.is_empty() {
                    app.textarea = None;
                    app.mode = AppMode::Normal;
                    return;
                }
                if let Err(msg) = Permission::validate(&input) {
                    app.status_message = Some(msg.to_string());
                    return;
                }
                match &app.mode {
                    AppMode::Adding => {
                        app.add_permission(input);
                        app.status_message = Some("Permission added".to_string());
                    }
                    AppMode::Editing { index } => {
                        app.edit_permission(*index, input);
                        app.status_message = Some("Permission updated".to_string());
                    }
                    _ => {}
                }
            }
            app.textarea = None;
            app.mode = AppMode::Normal;
        }
        _ => {
            if let Some(ref mut textarea) = app.textarea {
                textarea.input(key);
            }
        }
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

fn handle_moving_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if let AppMode::Moving { destinations, selected, .. } = &mut app.mode {
                if *selected + 1 < destinations.len() {
                    *selected += 1;
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let AppMode::Moving { selected, .. } = &mut app.mode {
                *selected = selected.saturating_sub(1);
            }
        }
        KeyCode::Char('u') | KeyCode::Char('p') | KeyCode::Char('l') => {
            let target = match key.code {
                KeyCode::Char('u') => SettingsSource::User,
                KeyCode::Char('p') => SettingsSource::Project,
                KeyCode::Char('l') => SettingsSource::Local,
                _ => unreachable!(),
            };
            if let AppMode::Moving { index, destinations, .. } = &app.mode {
                if let Some(pos) = destinations.iter().position(|s| *s == target) {
                    let index = *index;
                    let destination = destinations[pos];
                    app.move_permission(index, destination);
                    app.status_message = Some(format!("Moved to {}", destination.label()));
                    app.mode = AppMode::Normal;
                }
            }
        }
        KeyCode::Enter => {
            if let AppMode::Moving { index, destinations, selected, .. } = &app.mode {
                let index = *index;
                let destination = destinations[*selected];
                app.move_permission(index, destination);
                app.status_message = Some(format!("Moved to {}", destination.label()));
            }
            app.mode = AppMode::Normal;
        }
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
}

fn handle_changing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if let AppMode::Changing { destinations, selected, .. } = &mut app.mode {
                if *selected + 1 < destinations.len() {
                    *selected += 1;
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let AppMode::Changing { selected, .. } = &mut app.mode {
                *selected = selected.saturating_sub(1);
            }
        }
        KeyCode::Char('a') | KeyCode::Char('d') | KeyCode::Char('K') => {
            let target = match key.code {
                KeyCode::Char('a') => PermissionType::Allow,
                KeyCode::Char('d') => PermissionType::Deny,
                KeyCode::Char('K') => PermissionType::Ask,
                _ => unreachable!(),
            };
            if let AppMode::Changing { index, destinations, .. } = &app.mode {
                if destinations.contains(&target) {
                    let index = *index;
                    app.change_permission_type(index, target.clone());
                    app.status_message = Some(format!("Changed to {}", target));
                    app.mode = AppMode::Normal;
                }
            }
        }
        KeyCode::Enter => {
            if let AppMode::Changing { index, destinations, selected, .. } = &app.mode {
                let index = *index;
                let destination = destinations[*selected].clone();
                app.change_permission_type(index, destination.clone());
                app.status_message = Some(format!("Changed to {}", destination));
            }
            app.mode = AppMode::Normal;
        }
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
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
