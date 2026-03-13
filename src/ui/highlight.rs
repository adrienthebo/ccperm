use ratatui::{
    style::{Color, Style},
    text::Span,
};

const TOOL_STYLE: Style = Style::new().fg(Color::Green);
const PAREN_STYLE: Style = Style::new().fg(Color::DarkGray);
const WILDCARD_STYLE: Style = Style::new().fg(Color::Blue);
const LEGACY_WILDCARD_STYLE: Style = Style::new().fg(Color::Cyan);

#[derive(Debug, PartialEq)]
enum State {
    Tool,
    Specifier,
    ColonInSpecifier,
    Done,
}

pub fn highlight_permission(raw: &str) -> Vec<Span<'static>> {
    highlight(raw)
}

pub fn highlight_input(input: &str) -> Vec<Span<'static>> {
    highlight(input)
}

fn highlight(input: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut state = State::Tool;
    let mut buf = String::new();

    for ch in input.chars() {
        match state {
            State::Tool => {
                if ch == '(' {
                    if !buf.is_empty() {
                        spans.push(Span::styled(buf.clone(), TOOL_STYLE));
                        buf.clear();
                    }
                    spans.push(Span::styled("(", PAREN_STYLE));
                    state = State::Specifier;
                } else {
                    buf.push(ch);
                }
            }
            State::Specifier => {
                if ch == ')' {
                    flush_specifier(&mut buf, &mut spans);
                    spans.push(Span::styled(")", PAREN_STYLE));
                    state = State::Done;
                } else if ch == '*' {
                    flush_specifier(&mut buf, &mut spans);
                    spans.push(Span::styled("*", WILDCARD_STYLE));
                } else if ch == ':' {
                    state = State::ColonInSpecifier;
                } else {
                    buf.push(ch);
                }
            }
            State::ColonInSpecifier => {
                if ch == '*' {
                    // Legacy :* wildcard syntax
                    flush_specifier(&mut buf, &mut spans);
                    spans.push(Span::styled(":*", LEGACY_WILDCARD_STYLE));
                    state = State::Specifier;
                } else if ch == ')' {
                    buf.push(':');
                    flush_specifier(&mut buf, &mut spans);
                    spans.push(Span::styled(")", PAREN_STYLE));
                    state = State::Done;
                } else {
                    // Colon was just part of the specifier text
                    buf.push(':');
                    buf.push(ch);
                    state = State::Specifier;
                }
            }
            State::Done => {
                buf.push(ch);
            }
        }
    }

    // Flush remaining buffer
    if state == State::ColonInSpecifier {
        buf.push(':');
    }
    if !buf.is_empty() {
        let style = match state {
            State::Tool => TOOL_STYLE,
            State::Specifier | State::ColonInSpecifier => Style::default(),
            State::Done => Style::default(),
        };
        spans.push(Span::styled(buf, style));
    }

    spans
}

fn flush_specifier(buf: &mut String, spans: &mut Vec<Span<'static>>) {
    if !buf.is_empty() {
        spans.push(Span::raw(buf.clone()));
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(spans: &[Span]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn colors(spans: &[Span]) -> Vec<Option<Color>> {
        spans.iter().map(|s| s.style.fg).collect()
    }

    #[test]
    fn bare_tool() {
        let spans = highlight_permission("Bash");
        assert_eq!(text(&spans), "Bash");
        assert_eq!(colors(&spans), vec![Some(Color::Green)]);
    }

    #[test]
    fn tool_with_specifier() {
        let spans = highlight_permission("Bash(npm install)");
        assert_eq!(text(&spans), "Bash(npm install)");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),     // Bash
                Some(Color::DarkGray), // (
                None,                  // npm install
                Some(Color::DarkGray), // )
            ]
        );
    }

    #[test]
    fn wildcard_in_specifier() {
        let spans = highlight_permission("Bash(git * main)");
        assert_eq!(text(&spans), "Bash(git * main)");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),     // Bash
                Some(Color::DarkGray), // (
                None,                  // "git "
                Some(Color::Blue),  // *
                None,                  // " main"
                Some(Color::DarkGray), // )
            ]
        );
    }

    #[test]
    fn trailing_wildcard() {
        let spans = highlight_permission("Bash(npm run *)");
        assert_eq!(text(&spans), "Bash(npm run *)");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),     // Bash
                Some(Color::DarkGray), // (
                None,                  // "npm run "
                Some(Color::Blue),  // *
                Some(Color::DarkGray), // )
            ]
        );
    }

    #[test]
    fn mcp_tool() {
        let spans = highlight_permission("mcp__puppeteer__navigate");
        assert_eq!(text(&spans), "mcp__puppeteer__navigate");
        assert_eq!(colors(&spans), vec![Some(Color::Green)]);
    }

    #[test]
    fn incomplete_input() {
        let spans = highlight_input("Bash(npm ");
        assert_eq!(text(&spans), "Bash(npm ");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),     // Bash
                Some(Color::DarkGray), // (
                None,                  // "npm "
            ]
        );
    }

    #[test]
    fn empty_string() {
        let spans = highlight_permission("");
        assert!(spans.is_empty());
    }

    #[test]
    fn legacy_colon_wildcard() {
        let spans = highlight_permission("Bash(npm install:*)");
        assert_eq!(text(&spans), "Bash(npm install:*)");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),         // Bash
                Some(Color::DarkGray),     // (
                None,                      // "npm install"
                Some(Color::Cyan), // :*
                Some(Color::DarkGray),     // )
            ]
        );
    }

    #[test]
    fn colon_not_followed_by_wildcard() {
        let spans = highlight_permission("WebFetch(domain:example.com)");
        assert_eq!(text(&spans), "WebFetch(domain:example.com)");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),     // WebFetch
                Some(Color::DarkGray), // (
                None,                  // domain:example.com
                Some(Color::DarkGray), // )
            ]
        );
    }

    #[test]
    fn colon_at_end_of_input() {
        let spans = highlight_input("Bash(npm install:");
        assert_eq!(text(&spans), "Bash(npm install:");
        assert_eq!(
            colors(&spans),
            vec![
                Some(Color::Green),     // Bash
                Some(Color::DarkGray), // (
                None,                  // "npm install:"
            ]
        );
    }
}
