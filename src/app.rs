use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};

pub const TABS: [&str; 2] = ["Reminders", "Todo"];

pub struct Reminder {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub remind_at: Option<DateTime<Local>>,
    pub lead: Option<Duration>,
}

pub enum Screen {
    List,
    Detail,
}

pub enum Mode {
    Browse,
    QuickAdd(String),
    Edit {
        id: u32,
        focus: usize,
        buffer: [String; 4], // name, description, when, remind before
    },
    ConfirmDelete(u32),
}

pub enum ModeKind {
    Browse,
    QuickAdd,
    Edit,
    ConfirmDelete,
}

pub struct App {
    pub tab: usize,
    pub reminders: Vec<Reminder>,
    pub selected: usize,
    pub screen: Screen,
    pub mode: Mode,
    pub edit_error: Option<String>,
    next_id: u32,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let now = Local::now();
        // ponytail: seed demo data since there's no persistence yet; delete once reminders can be created and saved for real
        let reminders = vec![
            Reminder {
                id: 0,
                name: "Stand up".into(),
                description: Some("Daily sync with the team".into()),
                remind_at: Some(now + Duration::seconds(35)),
                lead: Some(Duration::minutes(5)),
            },
            Reminder {
                id: 1,
                name: "Dentist".into(),
                description: Some("Checkup appointment".into()),
                remind_at: Some(now + Duration::hours(2)),
                lead: Some(Duration::hours(1)),
            },
            Reminder {
                id: 2,
                name: "Pay rent".into(),
                description: Some("Bank transfer due".into()),
                remind_at: Some(now + Duration::days(3)),
                lead: Some(Duration::days(2)),
            },
            Reminder {
                id: 3,
                name: "Quick note".into(),
                description: None,
                remind_at: None,
                lead: None,
            },
        ];
        Self {
            tab: 0,
            reminders,
            selected: 0,
            screen: Screen::List,
            mode: Mode::Browse,
            edit_error: None,
            next_id: 4,
            should_quit: false,
        }
    }

    pub fn mode_kind(&self) -> ModeKind {
        match &self.mode {
            Mode::Browse => ModeKind::Browse,
            Mode::QuickAdd(_) => ModeKind::QuickAdd,
            Mode::Edit { .. } => ModeKind::Edit,
            Mode::ConfirmDelete(_) => ModeKind::ConfirmDelete,
        }
    }

    pub fn find_by_id(&self, id: u32) -> Option<&Reminder> {
        self.reminders.iter().find(|r| r.id == id)
    }

    pub fn sorted_reminders(&self) -> Vec<&Reminder> {
        let mut items: Vec<&Reminder> = self.reminders.iter().collect();
        // no-date reminders sort last
        items.sort_by_key(|r| r.remind_at.unwrap_or(DateTime::<Local>::MAX_UTC.into()));
        items
    }

    pub fn selected_reminder(&self) -> Option<&Reminder> {
        self.sorted_reminders().into_iter().nth(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.reminders.is_empty() {
            self.selected = (self.selected + 1).min(self.reminders.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn open_detail(&mut self) {
        if !self.reminders.is_empty() && matches!(self.mode, Mode::Browse) {
            self.screen = Screen::Detail;
        }
    }

    pub fn back_to_list(&mut self) {
        self.screen = Screen::List;
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % TABS.len();
    }

    pub fn prev_tab(&mut self) {
        self.tab = (self.tab + TABS.len() - 1) % TABS.len();
    }

    pub fn cancel_mode(&mut self) {
        self.mode = Mode::Browse;
        self.edit_error = None;
    }

    pub fn start_delete_confirm(&mut self) {
        if matches!(self.mode, Mode::Browse) {
            if let Some(r) = self.selected_reminder() {
                self.mode = Mode::ConfirmDelete(r.id);
            }
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Mode::ConfirmDelete(id) = self.mode {
            self.reminders.retain(|r| r.id != id);
            if self.selected >= self.reminders.len() {
                self.selected = self.reminders.len().saturating_sub(1);
            }
        }
        self.mode = Mode::Browse;
    }

    pub fn start_quick_add(&mut self) {
        if matches!(self.mode, Mode::Browse) {
            self.mode = Mode::QuickAdd(String::new());
        }
    }

    pub fn quick_add_push(&mut self, c: char) {
        if let Mode::QuickAdd(buf) = &mut self.mode {
            buf.push(c);
        }
    }

    pub fn quick_add_backspace(&mut self) {
        if let Mode::QuickAdd(buf) = &mut self.mode {
            buf.pop();
        }
    }

    pub fn confirm_quick_add(&mut self) {
        if let Mode::QuickAdd(buf) = &self.mode {
            let name = buf.trim().to_string();
            if !name.is_empty() {
                let id = self.next_id;
                self.next_id += 1;
                self.reminders.push(Reminder {
                    id,
                    name,
                    description: None,
                    remind_at: None,
                    lead: None,
                });
            }
        }
        self.mode = Mode::Browse;
    }

    pub fn start_edit(&mut self) {
        if !matches!(self.mode, Mode::Browse) {
            return;
        }
        let Some(r) = self.selected_reminder() else {
            return;
        };
        let buffer = [
            r.name.clone(),
            r.description.clone().unwrap_or_default(),
            datetime_to_input(r.remind_at),
            lead_to_input(r.lead),
        ];
        self.mode = Mode::Edit {
            id: r.id,
            focus: 0,
            buffer,
        };
        self.edit_error = None;
    }

    pub fn edit_next_field(&mut self) {
        if let Mode::Edit { focus, .. } = &mut self.mode {
            *focus = (*focus + 1) % 4;
        }
    }

    pub fn edit_prev_field(&mut self) {
        if let Mode::Edit { focus, .. } = &mut self.mode {
            *focus = (*focus + 3) % 4;
        }
    }

    pub fn edit_push_char(&mut self, c: char) {
        if let Mode::Edit { focus, buffer, .. } = &mut self.mode {
            buffer[*focus].push(c);
        }
    }

    pub fn edit_backspace(&mut self) {
        if let Mode::Edit { focus, buffer, .. } = &mut self.mode {
            buffer[*focus].pop();
        }
    }

    pub fn confirm_edit(&mut self) {
        let Mode::Edit { id, buffer, .. } = &self.mode else {
            return;
        };
        let name = buffer[0].trim().to_string();
        if name.is_empty() {
            self.edit_error = Some("Name can't be empty".into());
            return;
        }
        let description = non_empty(&buffer[1]);
        let when_raw = buffer[2].trim();
        let lead_raw = buffer[3].trim();

        let remind_at = if when_raw.is_empty() {
            None
        } else {
            match parse_datetime(when_raw) {
                Some(dt) => Some(dt),
                None => {
                    self.edit_error = Some("Couldn't parse date/time, use YYYY-MM-DD HH:MM".into());
                    return;
                }
            }
        };
        let lead = if lead_raw.is_empty() {
            None
        } else {
            match parse_lead(lead_raw) {
                Some(d) => Some(d),
                None => {
                    self.edit_error = Some("Couldn't parse lead time, use e.g. 2d, 12h, 30m".into());
                    return;
                }
            }
        };

        let id = *id;
        if let Some(r) = self.reminders.iter_mut().find(|r| r.id == id) {
            r.name = name;
            r.description = description;
            r.remind_at = remind_at;
            r.lead = lead;
        }
        self.mode = Mode::Browse;
        self.edit_error = None;
    }
}

fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn datetime_to_input(dt: Option<DateTime<Local>>) -> String {
    dt.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default()
}

fn lead_to_input(lead: Option<Duration>) -> String {
    lead.map(format_lead_raw).unwrap_or_default()
}

fn format_lead_raw(lead: Duration) -> String {
    let secs = lead.num_seconds();
    if secs % 86400 == 0 {
        format!("{}d", secs / 86400)
    } else if secs % 3600 == 0 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}m", secs / 60)
    }
}

fn parse_datetime(s: &str) -> Option<DateTime<Local>> {
    let formats = ["%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"];
    for fmt in formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
            if let Some(dt) = Local.from_local_datetime(&naive).single() {
                return Some(dt);
            }
        }
    }
    None
}

fn parse_lead(s: &str) -> Option<Duration> {
    let s = s.trim();
    let unit = s.chars().last()?;
    let number: i64 = s[..s.len() - unit.len_utf8()].trim().parse().ok()?;
    match unit {
        'd' | 'D' => Some(Duration::days(number)),
        'h' | 'H' => Some(Duration::hours(number)),
        'm' | 'M' => Some(Duration::minutes(number)),
        _ => None,
    }
}

pub fn format_lead(lead: Option<Duration>) -> String {
    match lead {
        Some(d) => format_lead_raw(d),
        None => "-".to_string(),
    }
}

pub fn format_datetime(target: Option<DateTime<Local>>) -> String {
    match target {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "-".to_string(),
    }
}

pub fn relative_time(target: Option<DateTime<Local>>) -> String {
    let Some(target) = target else {
        return "-".to_string();
    };

    let diff = target - Local::now();
    let past = diff < Duration::zero();
    let secs = diff.num_seconds().abs();

    let body = if secs < 60 {
        format!("{secs}s")
    } else if secs < 300 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 3600 {
        pluralize(secs / 60, "min", "min")
    } else if secs < 86400 {
        pluralize(secs / 3600, "hour", "hours")
    } else {
        pluralize(secs / 86400, "day", "days")
    };

    if past {
        format!("{body} ago")
    } else {
        format!("in {body}")
    }
}

fn pluralize(n: i64, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("1 {singular}")
    } else {
        format!("{n} {plural}")
    }
}
