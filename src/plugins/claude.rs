use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::format;
use crate::macos;
use crate::plugins::{Item, Output, Plugin};

const PLACEHOLDER: &str = "\u{2013}";
const FETCH_EVERY: u64 = 30;
const UNREACHABLE: &str = "unreachable";
const MAX_STALE_AGE: Duration = Duration::from_secs(300);

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const WINDOWS: [(&str, &str, &str, &str); 3] = [
    ("session", "five_hour", "session", "Session"),
    ("weekly_all", "seven_day", "week", "Week (all models)"),
    ("weekly_scoped", "seven_day_opus", "Fable", "Week (Fable)"),
];

pub struct Claude {
    shared: Arc<Mutex<FetchSlot>>,
    samples: u64,
}

impl Claude {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(FetchSlot {
                fetched: None,
                last_ok: None,
                in_flight: false,
            })),
            samples: 0,
        }
    }
}

impl Plugin for Claude {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn label(&self) -> &'static str {
        "Claude"
    }

    fn prefix(&self) -> &'static str {
        "A"
    }

    fn sample(&mut self) -> Output {
        self.fetch_if_due();
        self.samples += 1;

        let shared = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        let fetched = match (&shared.fetched, &shared.last_ok) {
            (Some(Err(UNREACHABLE)), Some((usage, at))) if at.elapsed() < MAX_STALE_AGE => {
                Some(Ok(usage.clone()))
            }
            (fetched, _) => fetched.clone(),
        };
        drop(shared);

        let (segment, tooltip_lines, items) = match fetched {
            Some(Ok(usage)) => (
                usage[0]
                    .as_ref()
                    .map(|w| format::percent_f64(w.used_percent)),
                vec![usage_tooltip(&usage)],
                usage_items(&usage),
            ),
            Some(Err(reason)) => (
                None,
                vec![format!("Claude: {reason}")],
                vec![Item {
                    text: "Claude usage unavailable \u{2014} sign in with Claude Code".to_string(),
                    badge: None,
                }],
            ),
            None => (None, Vec::new(), Vec::new()),
        };

        Output {
            segment: Some(segment.unwrap_or_else(|| PLACEHOLDER.to_string())),
            tooltip_lines,
            menu_items: Some(items),
        }
    }

    fn invalidate(&mut self) {
        self.samples = 0;
    }
}

impl Claude {
    fn fetch_if_due(&self) {
        let mut shared = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        if !self.samples.is_multiple_of(FETCH_EVERY) || shared.in_flight {
            return;
        }
        shared.in_flight = true;
        let shared = Arc::clone(&self.shared);
        std::thread::spawn(move || {
            let fetched = fetch();
            let mut shared = shared.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(usage) = &fetched {
                shared.last_ok = Some((usage.clone(), Instant::now()));
            }
            shared.fetched = Some(fetched);
            shared.in_flight = false;
        });
    }
}

struct FetchSlot {
    fetched: Fetched,
    last_ok: Option<(Usage, Instant)>,
    in_flight: bool,
}

type Fetched = Option<Result<Usage, &'static str>>;

type Usage = [Option<LimitWindow>; 3];

#[derive(Clone)]
struct LimitWindow {
    used_percent: f64,
    resets_at: Option<i64>,
}

fn fetch() -> Result<Usage, &'static str> {
    let token = macos::claude_access_token().ok_or("no credentials")?;
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build(),
    );
    let result = agent
        .get(USAGE_URL)
        .header("Authorization", format!("Bearer {token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .call();
    match result {
        Ok(mut response) => serde_json::from_reader(response.body_mut().as_reader())
            .ok()
            .and_then(|value| parse_usage(&value))
            .ok_or(UNREACHABLE),
        Err(ureq::Error::StatusCode(401 | 403)) => Err("authorization failed"),
        Err(_) => Err(UNREACHABLE),
    }
}

fn parse_usage(value: &serde_json::Value) -> Option<Usage> {
    let limits = value.get("limits").and_then(|v| v.as_array());
    let usage = WINDOWS.map(|(kind, legacy, _, _)| {
        limits
            .into_iter()
            .flatten()
            .filter(|limit| limit.get("kind").and_then(|v| v.as_str()) == Some(kind))
            .filter_map(|limit| parse_window(limit, "percent"))
            .next_back()
            .or_else(|| parse_window(value.get(legacy)?, "utilization"))
    });
    usage.iter().any(Option::is_some).then_some(usage)
}

fn parse_window(value: &serde_json::Value, key: &str) -> Option<LimitWindow> {
    Some(LimitWindow {
        used_percent: value.get(key)?.as_f64()?,
        resets_at: value.get("resets_at").and_then(parse_resets_at),
    })
}

fn parse_resets_at(value: &serde_json::Value) -> Option<i64> {
    if let Some(epoch) = value.as_i64() {
        return Some(epoch);
    }
    OffsetDateTime::parse(value.as_str()?, &Rfc3339)
        .ok()
        .map(|t| t.unix_timestamp())
}

fn usage_tooltip(usage: &Usage) -> String {
    let parts: Vec<String> = WINDOWS
        .iter()
        .zip(usage)
        .filter_map(|((_, _, name, _), window)| {
            Some(format!(
                "{name} {}",
                format::percent_f64(window.as_ref()?.used_percent)
            ))
        })
        .collect();
    format!("Claude: {}", parts.join(" \u{00b7} "))
}

fn usage_items(usage: &Usage) -> Vec<Item> {
    WINDOWS
        .iter()
        .zip(usage)
        .filter_map(|((_, _, _, label), window)| {
            let window = window.as_ref()?;
            Some(Item {
                text: format!("{label}{}", resets_label(window.resets_at)),
                badge: Some(format::percent_f64(window.used_percent)),
            })
        })
        .collect()
}

fn resets_label(resets_at: Option<i64>) -> String {
    let Some(resets_at) = resets_at else {
        return String::new();
    };
    let delta = resets_at - OffsetDateTime::now_utc().unix_timestamp();
    if delta <= 0 {
        return " \u{00b7} resets soon".to_string();
    }
    let minutes = delta / 60;
    if minutes < 60 {
        format!(" \u{00b7} resets in {minutes}m")
    } else if minutes < 24 * 60 {
        format!(" \u{00b7} resets in {}h {:02}m", minutes / 60, minutes % 60)
    } else {
        format!(" \u{00b7} resets in {}d", minutes / (24 * 60))
    }
}
