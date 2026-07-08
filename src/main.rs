mod app;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};

use app::{App, Screen};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // ponytail: 250ms poll so the countdown keeps ticking without a full redraw thread
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key);
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::List => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
            KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
            KeyCode::Char('a') if app.tab == 0 => app.open_add_form(),
            _ => {}
        },
        Screen::Add => match key.code {
            KeyCode::Esc => app.cancel_add_form(),
            KeyCode::Tab | KeyCode::Down => app.form_next_field(),
            KeyCode::BackTab | KeyCode::Up => app.form_prev_field(),
            KeyCode::Enter => app.submit_form(),
            KeyCode::Backspace => {
                app.form.fields[app.form.focused].pop();
            }
            KeyCode::Char(c) => app.form.fields[app.form.focused].push(c),
            _ => {}
        },
    }
}
