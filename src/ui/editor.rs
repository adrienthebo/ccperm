use crate::app::{App, AppMode};
use crate::ui::layout::centered_rect;
use super::highlight::highlight_input;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_editor(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 30, frame.area());

    let (title, input) = match &app.mode {
        AppMode::Adding { input } => ("Add Permission", input.as_str()),
        AppMode::Editing { input, .. } => ("Edit Permission", input.as_str()),
        _ => return,
    };

    let text = vec![
        Line::from(""),
        Line::from("Enter permission rule:"),
        Line::from(""),
        Line::from({
            let mut spans = vec![Span::raw("> ")];
            spans.extend(highlight_input(input));
            spans.push(Span::styled("_", Style::default().fg(Color::Gray)));
            spans
        }),
        Line::from(""),
        Line::from("Examples:"),
        Line::from(Span::styled(
            "  Bash(npm install:*)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Bash(git commit:*)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  WebFetch(domain:github.com)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];

    let dialog = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title))
            .style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(dialog, area);
}
