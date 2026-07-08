use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Paragraph, Row, Table, Tabs};
use ratatui::Frame;

use crate::app::{format_datetime, format_lead, relative_time, App, Mode, Screen, TABS};

pub fn draw(frame: &mut Frame, app: &App) {
    let [content, tab_bar] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

    match app.screen {
        Screen::List => draw_content(frame, app, content),
        Screen::Detail => draw_detail(frame, app, content),
    }
    draw_tab_bar(frame, app, tab_bar);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    if app.tab == 0 {
        draw_reminders(frame, app, area);
    } else {
        frame.render_widget(Paragraph::new("Coming soon").alignment(Alignment::Center), area);
    }
}

fn draw_reminders(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Date & Time").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Remind At").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let editing_id = match &app.mode {
        Mode::Edit { id, .. } => Some(*id),
        _ => None,
    };

    let mut rows: Vec<Row> = app
        .sorted_reminders()
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            if editing_id == Some(r.id) {
                let Mode::Edit { focus, buffer, .. } = &app.mode else {
                    unreachable!()
                };
                return Row::new(vec![
                    field_cell(&buffer[0], *focus == 0),
                    field_cell(&buffer[2], *focus == 2),
                    field_cell(&buffer[3], *focus == 3),
                ])
                .style(Style::default().fg(Color::Yellow));
            }

            let selected = i == app.selected && matches!(app.mode, Mode::Browse);
            let style = if selected {
                Style::default().bg(Color::Cyan).fg(Color::Black)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(r.name.clone()),
                Cell::from(format_datetime(r.remind_at)),
                Cell::from(relative_time(r.remind_at)),
            ])
            .style(style)
        })
        .collect();

    if let Mode::QuickAdd(buffer) = &app.mode {
        rows.push(
            Row::new(vec![field_cell(buffer, true), Cell::from("-"), Cell::from("-")])
                .style(Style::default().fg(Color::Yellow)),
        );
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(24),
            Constraint::Length(18),
            Constraint::Min(12),
        ],
    )
    .header(header);

    frame.render_widget(table, area);

    if let Mode::Edit { buffer, focus, .. } = &app.mode {
        let mut lines = vec![Line::from(format!(
            "Description: {}{}",
            buffer[1],
            if *focus == 1 { "_" } else { "" }
        ))];
        if let Some(err) = &app.edit_error {
            lines.push(Line::from(err.clone()).style(Style::default().fg(Color::Red)));
        }
        let below = Rect {
            y: area.y + area.height.saturating_sub(lines.len() as u16 + 1),
            height: lines.len() as u16 + 1,
            ..area
        };
        frame.render_widget(Paragraph::new(lines), below);
    }
}

fn field_cell(value: &str, focused: bool) -> Cell<'static> {
    let text = if focused {
        format!("{value}_")
    } else {
        value.to_string()
    };
    Cell::from(text)
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(reminder) = app.selected_reminder() else {
        frame.render_widget(Paragraph::new("No reminder selected"), area);
        return;
    };

    let lines = vec![
        Line::from(reminder.name.clone()).style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from(format!(
            "Description: {}",
            reminder.description.as_deref().unwrap_or("-")
        )),
        Line::from(format!("Date & Time: {}", format_datetime(reminder.remind_at))),
        Line::from(format!("Remind At: {}", relative_time(reminder.remind_at))),
        Line::from(format!("Remind before: {}", format_lead(reminder.lead))),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let tabs = Tabs::new(TABS.to_vec())
        .select(app.tab)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .divider(" ");
    frame.render_widget(tabs, area);
}
