use crate::app::{App, FlatItem};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render_tree(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.build_flat_items();

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| match item {
            FlatItem::Category {
                category,
                count,
                expanded,
            } => {
                let icon = if *expanded { "▼" } else { "▶" };
                let text = format!("{} {} ({})", icon, category, count);
                ListItem::new(Line::from(vec![Span::raw(text)]))
            }
            FlatItem::Permission { permission, .. } => {
                let display = permission.display_short();
                let text = format!("  ├─ {}", display);
                ListItem::new(Line::from(vec![Span::raw(text)]))
            }
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Permissions "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    if !items.is_empty() {
        list_state.select(Some(app.tree_state.flat_index));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}
