use crate::app::{App, AppMode, SettingsSource};
use crate::config::PermissionType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use super::editor::render_editor;
use super::help::render_help;
use super::picker::render_picker;
use super::tree::render_tree;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(5),    // Tree
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);
    render_tabs(frame, chunks[1], app);
    render_tree(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);

    // Render modal dialogs
    match &app.mode {
        AppMode::Adding { .. } | AppMode::Editing { .. } => {
            render_editor(frame, app);
        }
        AppMode::Help => {
            render_help(frame);
        }
        AppMode::Confirm { message, .. } => {
            render_confirm(frame, message);
        }
        AppMode::Moving { .. } => {
            render_picker(frame, app);
        }
        AppMode::Normal => {}
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![Span::styled(" ccperm", Style::default().fg(Color::Cyan))];

    let sources: Vec<(SettingsSource, &str, &str)> = if app.project_root.is_some() {
        vec![
            (SettingsSource::User, "[U]", "ser"),
            (SettingsSource::Project, "[P]", "roject"),
            (SettingsSource::Local, "[L]", "ocal"),
        ]
    } else {
        vec![(SettingsSource::User, "[U]", "ser")]
    };

    for (source, key, rest) in &sources {
        spans.push(Span::raw("  "));
        let is_selected = *source == app.selected_source;
        let (added, removed) = app.source_changes(*source);

        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let key_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };

        spans.push(Span::styled(*key, key_style));
        spans.push(Span::styled(*rest, style));

        if added > 0 || removed > 0 {
            let mut parts = Vec::new();
            if added > 0 {
                parts.push(format!("+{}", added));
            }
            if removed > 0 {
                parts.push(format!("-{}", removed));
            }
            spans.push(Span::styled(
                format!(" [{}]", parts.join("/")),
                Style::default().fg(Color::Red),
            ));
        }
    }

    spans.push(Span::styled("  [?] Help  [q] Quit ", Style::default().fg(Color::Cyan)));

    let header = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let tab_titles = vec![
        format!("Allow ({})", app.current_settings().permissions.allow.len()),
        format!("Deny ({})", app.current_settings().permissions.deny.len()),
        format!("Ask ({})", app.current_settings().permissions.ask.len()),
    ];

    let selected = match app.selected_tab {
        PermissionType::Allow => 0,
        PermissionType::Deny => 1,
        PermissionType::Ask => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" [Tab] Switch "));

    frame.render_widget(tabs, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let status = app.status_message.as_deref().unwrap_or("");

    let help_text = Line::from(vec![
        Span::styled("[a]", Style::default().fg(Color::Yellow)),
        Span::raw("dd "),
        Span::styled("[e]", Style::default().fg(Color::Yellow)),
        Span::raw("dit "),
        Span::styled("[d]", Style::default().fg(Color::Yellow)),
        Span::raw("elete "),
        Span::styled("[m]", Style::default().fg(Color::Yellow)),
        Span::raw("ove s"),
        Span::styled("[o]", Style::default().fg(Color::Yellow)),
        Span::raw("rt "),
        Span::styled("[s]", Style::default().fg(Color::Yellow)),
        Span::raw("ave "),
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw("eload "),
        Span::styled("[u]", Style::default().fg(Color::Yellow)),
        Span::raw("ser "),
        Span::styled("[p]", Style::default().fg(Color::Yellow)),
        Span::raw("roject "),
        Span::styled("[l]", Style::default().fg(Color::Yellow)),
        Span::raw("ocal  "),
        Span::styled(status, Style::default().fg(Color::Green)),
    ]);

    let total = app.current_permissions().len();
    let count_text = format!("Total: {}", total);

    let footer = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", count_text)),
    );

    frame.render_widget(footer, area);
}

fn render_confirm(frame: &mut Frame, message: &str) {
    let area = centered_rect(50, 20, frame.area());

    let text = vec![
        Line::from(""),
        Line::from(message),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y]", Style::default().fg(Color::Green)),
            Span::raw(" Yes  "),
            Span::styled("[n]", Style::default().fg(Color::Red)),
            Span::raw(" No"),
        ]),
    ];

    let dialog = Paragraph::new(text)
        .style(Style::default())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm ")
                .style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(dialog, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
