use crate::app::{App, AppMode};
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_editor(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(70, 40, frame.area());

    let title = match &app.mode {
        AppMode::Adding => " Add Permission ",
        AppMode::Editing { .. } => " Edit Permission ",
        _ => return,
    };

    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Cyan)),
        area,
    );

    let inner = Block::default().borders(Borders::ALL).inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // "Enter permission rule:" + blank
            Constraint::Length(1), // TextArea
            Constraint::Length(1), // blank
            Constraint::Min(1),   // Examples
            Constraint::Length(2), // blank + footer
        ])
        .split(inner);

    let prompt = Paragraph::new("Enter permission rule:");
    frame.render_widget(prompt, chunks[0]);

    if let Some(ref mut textarea) = app.textarea {
        textarea.set_cursor_line_style(Style::default());
        textarea.set_cursor_style(Style::default().fg(Color::Gray).bg(Color::DarkGray));
        frame.render_widget(&*textarea, chunks[1]);
    }

    let examples = vec![
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
    ];
    frame.render_widget(Paragraph::new(examples), chunks[3]);

    let footer = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];
    frame.render_widget(Paragraph::new(footer), chunks[4]);
}
