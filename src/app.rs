use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};

pub const TABS: [&str; 2] = ["Reminders", "Todo"];

pub struct Reminder {
    pub name: String,
    pub description: String,
    pub remind_at: DateTime<Local>,
    pub lead: Duration,
}

pub enum Screen {
    List,
    Add,
}

#[derive(Default)]
pub struct FormState {
    pub fields: [String; 4], // name, description, when, lead
    pub focused: usize,
    pub error: Option<String>,
}

impl FormState {
    pub const LABELS: [&'static str; 4] = [
        "Name",
        "Description",
        "When (YYYY-MM-DD HH:MM)",
        "Remind before (e.g. 2d, 12h, 30m)",
    ];

    fn clear(&mut self) {
        self.fields = Default::default();
        self.focused = 0;
        self.error = None;
    }
}

pub struct App {
    pub tab: usize,
    pub reminders: Vec<Reminder>,
    pub screen: Screen,
    pub form: FormState,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let now = Local::now();
        // ponytail: seed demo data since there's no persistence yet; delete once reminders can be created and saved for real
        let reminders = vec![
            Reminder {
                name: "Stand up".into(),
                description: "Daily sync with the team".into(),
                remind_at: now + Duration::seconds(35),
                lead: Duration::minutes(5),
            },
            Reminder {
                name: "Dentist".into(),
                description: "Checkup appointment".into(),
                remind_at: now + Duration::hours(2),
                lead: Duration::hours(1),
            },
            Reminder {
                name: "Pay rent".into(),
                description: "Bank transfer due".into(),
                remind_at: now + Duration::days(3),
                lead: Duration::days(2),
            },
        ];
        Self {
            tab: 0,
            reminders,
            screen: Screen::List,
            form: FormState::default(),
            should_quit: false,
        }
    }

    pub fn sorted_reminders(&self) -> Vec<&Reminder> {
        let mut items: Vec<&Reminder> = self.reminders.iter().collect();
        items.sort_by_key(|r| r.remind_at);
        items
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % TABS.len();
    }

    pub fn prev_tab(&mut self) {
        self.tab = (self.tab + TABS.len() - 1) % TABS.len();
    }

    pub fn open_add_form(&mut self) {
        self.form.clear();
        self.screen = Screen::Add;
    }

    pub fn cancel_add_form(&mut self) {
        self.form.clear();
        self.screen = Screen::List;
    }

    pub fn form_next_field(&mut self) {
        self.form.focused = (self.form.focused + 1) % self.form.fields.len();
    }

    pub fn form_prev_field(&mut self) {
        self.form.focused =
            (self.form.focused + self.form.fields.len() - 1) % self.form.fields.len();
    }

    pub fn submit_form(&mut self) {
        let name = self.form.fields[0].trim().to_string();
        let description = self.form.fields[1].trim().to_string();
        let when_raw = self.form.fields[2].trim();
        let lead_raw = self.form.fields[3].trim();

        if name.is_empty() {
            self.form.error = Some("Name can't be empty".into());
            return;
        }
        let Some(remind_at) = parse_datetime(when_raw) else {
            self.form.error = Some("Couldn't parse date/time, use YYYY-MM-DD HH:MM".into());
            return;
        };
        let lead = if lead_raw.is_empty() {
            Duration::zero()
        } else {
            match parse_lead(lead_raw) {
                Some(d) => d,
                None => {
                    self.form.error = Some("Couldn't parse lead time, use e.g. 2d, 12h, 30m".into());
                    return;
                }
            }
        };

        self.reminders.push(Reminder {
            name,
            description,
            remind_at,
            lead,
        });
        self.cancel_add_form();
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

pub fn relative_time(target: DateTime<Local>) -> String {
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
