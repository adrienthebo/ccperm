use crate::app::{App, FlatItem};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_tree(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.build_flat_items();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == app.tree_state.flat_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            match item {
                FlatItem::Category {
                    category,
                    count,
                    expanded,
                } => {
                    let icon = if *expanded { "▼" } else { "▶" };
                    let text = format!("{} {} ({})", icon, category, count);
                    ListItem::new(Line::from(vec![Span::styled(text, style)]))
                }
                FlatItem::Permission { permission, .. } => {
                    let display = permission.display_short();
                    let text = format!("  ├─ {}", display);
                    ListItem::new(Line::from(vec![Span::styled(text, style)]))
                }
            }
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Permissions "),
    );

    frame.render_widget(list, area);
}
