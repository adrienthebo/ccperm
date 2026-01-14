mod app;
mod config;
mod event;
mod tui;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialize terminal
    let mut terminal = tui::init()?;

    // Create app state
    let mut app = app::App::new()?;

    // Main loop
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &app))?;
        event::handle_event(&mut app)?;
    }

    // Restore terminal
    tui::restore()?;

    Ok(())
}
