use crate::app::{App, DuplicateKind, FlatItem};
use super::highlight::highlight_permission;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render_tree(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.build_flat_items();
    let duplicates = app.detect_duplicates();

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
                let mut spans = vec![Span::raw("  ├─ ")];
                spans.extend(highlight_permission(&permission.raw));
                if let Some(kinds) = duplicates.get(&permission.raw) {
                    spans.push(Span::styled(
                        format_duplicate_label(kinds),
                        Style::default().fg(Color::Red),
                    ));
                }
                ListItem::new(Line::from(spans))
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

fn format_duplicate_label(kinds: &[DuplicateKind]) -> String {
    let mut labels = Vec::new();

    for kind in kinds {
        match kind {
            DuplicateKind::Duplicate { source } => {
                labels.push(format!(" [Duplicates {}]", source.label()));
            }
            DuplicateKind::Overrides { source, permission_type } => {
                labels.push(format!(" [Overrides {}/{}]", source.label(), permission_type));
            }
            DuplicateKind::OverriddenBy { source, permission_type } => {
                labels.push(format!(" [Overridden by {}/{}]", source.label(), permission_type));
            }
        }
    }

    labels.join("")
}
