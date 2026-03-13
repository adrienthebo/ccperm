use crate::app::{App, AppMode};
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
        .map(|s| ListItem::new(format!("  {}", s.label())))
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

    let footer = Line::from(vec![
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

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
