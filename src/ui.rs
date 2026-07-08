use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{format_lead, relative_time, App, Screen, TABS};

pub fn draw(frame: &mut Frame, app: &App) {
    let [content, tab_bar] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());

    match app.screen {
        Screen::List => draw_content(frame, app, content),
        Screen::Detail => draw_detail(frame, app, content),
        Screen::Add => draw_add_form(frame, app, content),
    }
    draw_tab_bar(frame, app, tab_bar);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    if app.tab == 0 {
        draw_reminders(frame, app, area);
    } else {
        let block = Block::default().borders(Borders::ALL).title("Todo");
        frame.render_widget(
            Paragraph::new("Coming soon").alignment(Alignment::Center).block(block),
            area,
        );
    }
}

fn draw_reminders(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Reminders")
        .title_bottom(Line::from(" j/k: move  l: open  a: add  q: quit ").right_aligned());

    let items: Vec<ListItem> = app
        .sorted_reminders()
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let selected = i == app.selected;
            let style = if selected {
                Style::default().bg(Color::Cyan).fg(Color::Black)
            } else {
                Style::default()
            };
            let line = Line::from(vec![
                Span::styled(format!("{:<24}", r.name), style.add_modifier(Modifier::BOLD)),
                Span::styled(relative_time(r.remind_at), style.fg(if selected { Color::Black } else { Color::Cyan })),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = if items.is_empty() {
        List::new(vec![ListItem::new("No reminders yet, press 'a' to add one")]).block(block)
    } else {
        List::new(items).block(block)
    };

    frame.render_widget(list, area);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Reminder")
        .title_bottom(Line::from(" h: back  q: quit ").right_aligned());

    let Some(reminder) = app.selected_reminder() else {
        frame.render_widget(Paragraph::new("No reminder selected").block(block), area);
        return;
    };

    let lines = vec![
        Line::from(Span::styled(reminder.name.clone(), Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!(
            "Description: {}",
            reminder.description.as_deref().unwrap_or("no description")
        )),
        Line::from(format!("When: {}", relative_time(reminder.remind_at))),
        Line::from(format!("Remind before: {}", format_lead(reminder.lead))),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_add_form(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(60, 60, area);

    let mut lines = vec![Line::from(""), Line::from("New Reminder"), Line::from("")];
    for (i, label) in crate::app::FormState::LABELS.iter().enumerate() {
        let focused = i == app.form.focused;
        let marker = if focused { "> " } else { "  " };
        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(format!("{marker}{label}:"), style)));
        let value = if focused {
            format!("  {}_", app.form.fields[i])
        } else {
            format!("  {}", app.form.fields[i])
        };
        lines.push(Line::from(value));
        lines.push(Line::from(""));
    }

    if let Some(err) = &app.form.error {
        lines.push(Line::from(Span::styled(err.clone(), Style::default().fg(Color::Red))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Add Reminder")
        .title_bottom(Line::from(" Tab: next  Enter: submit  Esc: cancel ").right_aligned());

    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let tabs = Tabs::new(TABS.to_vec())
        .block(Block::default().borders(Borders::ALL))
        .select(app.tab)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .divider(" ");
    frame.render_widget(tabs, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    area
}
