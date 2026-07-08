mod app;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};

use app::{App, ModeKind, Screen};

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
        Screen::List => match app.mode_kind() {
            ModeKind::Browse => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                KeyCode::Left => app.prev_tab(),
                KeyCode::Right => app.next_tab(),
                KeyCode::Char('j') | KeyCode::Down if app.tab == 0 => app.select_next(),
                KeyCode::Char('k') | KeyCode::Up if app.tab == 0 => app.select_prev(),
                KeyCode::Char('l') if app.tab == 0 => app.open_detail(),
                KeyCode::Char('a') if app.tab == 0 => app.start_quick_add(),
                KeyCode::Char('e') if app.tab == 0 => app.start_edit(),
                _ => {}
            },
            ModeKind::QuickAdd => match key.code {
                KeyCode::Enter => app.confirm_quick_add(),
                KeyCode::Esc => app.cancel_mode(),
                KeyCode::Backspace => app.quick_add_backspace(),
                KeyCode::Char(c) => app.quick_add_push(c),
                _ => {}
            },
            ModeKind::Edit => match key.code {
                KeyCode::Enter => app.confirm_edit(),
                KeyCode::Esc => app.cancel_mode(),
                KeyCode::Tab | KeyCode::Down => app.edit_next_field(),
                KeyCode::BackTab | KeyCode::Up => app.edit_prev_field(),
                KeyCode::Backspace => app.edit_backspace(),
                KeyCode::Char(c) => app.edit_push_char(c),
                _ => {}
            },
        },
        Screen::Detail => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('h') | KeyCode::Esc => app.back_to_list(),
            _ => {}
        },
    }
}
