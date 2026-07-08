use std::io;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Flex, Layout};
use ratatui::widgets::Paragraph;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|frame| {
            let [area] = Layout::vertical([Constraint::Length(1)])
                .flex(Flex::Center)
                .areas(frame.area());
            frame.render_widget(
                Paragraph::new("Hello, world!").alignment(Alignment::Center),
                area,
            );
        })?;

        if let Event::Key(key) = event::read()? {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                break;
            }
        }
    }

    ratatui::restore();
    Ok(())
}
