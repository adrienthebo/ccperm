use crate::app::{App, AppMode, SettingsSource};
use crate::config::PermissionType;
use crate::ui::layout::centered_rect;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn render_picker(frame: &mut Frame, app: &App) {
    let (permission, destinations, selected) = match &app.mode {
        AppMode::Moving {
            permission,
            destinations,
            selected,
            ..
        } => (permission.as_str(), destinations, *selected),
        _ => return,
    };

    let area = centered_rect(50, 30, frame.area());

    let items: Vec<ListItem> = destinations
        .iter()
        .map(|s| {
            let (key, rest) = match s {
                SettingsSource::User => ("[u]", "ser"),
                SettingsSource::Project => ("[p]", "roject"),
                SettingsSource::Local => ("[l]", "ocal"),
            };
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(key, Style::default().fg(Color::Yellow)),
                Span::raw(rest),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Move: {} ", truncate(permission, 30)))
                .style(Style::default().fg(Color::Cyan)),
        )
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let shortcut_keys: Vec<&str> = destinations
        .iter()
        .map(|s| match s {
            SettingsSource::User => "u",
            SettingsSource::Project => "p",
            SettingsSource::Local => "l",
        })
        .collect();
    let footer = Line::from(vec![
        Span::styled(
            format!("[{}]", shortcut_keys.join("/")),
            Style::default().fg(Color::Green),
        ),
        Span::raw("/"),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Move  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut list_state);

    // Render footer hint below the list area
    let footer_area = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(2),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    frame.render_widget(ratatui::widgets::Paragraph::new(footer), footer_area);
}

pub fn render_type_picker(frame: &mut Frame, app: &App) {
    let (permission, destinations, selected) = match &app.mode {
        AppMode::Changing {
            permission,
            destinations,
            selected,
            ..
        } => (permission.as_str(), destinations, *selected),
        _ => return,
    };

    let area = centered_rect(50, 30, frame.area());

    let items: Vec<ListItem> = destinations
        .iter()
        .map(|t| {
            let spans: Vec<Span> = match t {
                PermissionType::Allow => vec![
                    Span::styled("[a]", Style::default().fg(Color::Yellow)),
                    Span::raw("llow"),
                ],
                PermissionType::Deny => vec![
                    Span::styled("[d]", Style::default().fg(Color::Yellow)),
                    Span::raw("eny"),
                ],
                PermissionType::Ask => vec![
                    Span::raw("as"),
                    Span::styled("[K]", Style::default().fg(Color::Yellow)),
                ],
            };
            let mut line_spans = vec![Span::raw("  ")];
            line_spans.extend(spans);
            ListItem::new(Line::from(line_spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Change: {} ", truncate(permission, 30)))
                .style(Style::default().fg(Color::Cyan)),
        )
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let shortcut_keys: Vec<&str> = destinations
        .iter()
        .map(|t| match t {
            PermissionType::Allow => "a",
            PermissionType::Deny => "d",
            PermissionType::Ask => "K",
        })
        .collect();
    let footer = Line::from(vec![
        Span::styled(
            format!("[{}]", shortcut_keys.join("/")),
            Style::default().fg(Color::Green),
        ),
        Span::raw("/"),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Change  "),
        Span::styled("[Esc]", Style::default().fg(Color::Red)),
        Span::raw(" Cancel"),
    ]);

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut list_state);

    let footer_area = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(2),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    frame.render_widget(ratatui::widgets::Paragraph::new(footer), footer_area);
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
