use crate::ui::layout::centered_rect;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("j / ↓", "Move down"),
        help_line("k / ↑", "Move up"),
        help_line("h / ←", "Collapse category"),
        help_line("l / → / Enter", "Expand category"),
        help_line("Tab", "Switch tab (Allow/Deny/Ask)"),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("a", "Add new permission"),
        help_line("e", "Edit selected permission"),
        help_line("d", "Delete selected permission"),
        help_line("m", "Move permission to another source"),
        help_line("s", "Save changes"),
        help_line("r", "Reload from file"),
        Line::from(""),
        Line::from(Span::styled(
            "Settings",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("u", "Switch to User settings"),
        help_line("p", "Switch to Project settings"),
        help_line("l", "Switch to Local settings"),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        help_line("?", "Toggle this help"),
        help_line("q / Esc", "Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Press any key to close",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    let dialog = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(dialog, area);
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:15}", key),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(desc),
    ])
}
