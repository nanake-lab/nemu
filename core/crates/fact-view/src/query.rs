//! In-process plans only. IPC requests/responses use generated IDL types.
use core_types::generated::{EventKind, QueryFilter};
use std::collections::BTreeSet;

pub const QUERY_VERSION: &str = "w19-rrf-v1";
pub const INDEX_VERSION: &str = "w19-trigram-vec-v1";
pub(crate) const LIMITS: core_types::contract::Limits = core_types::contract::Limits {
    max_bytes: 4 * 1024 * 1024,
    max_items: 100_000,
    max_depth: 64,
};
pub(crate) fn kind_name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::STARTUP => "STARTUP",
        EventKind::OBSERVATION => "OBSERVATION",
        EventKind::ACTION_REQUEST => "ACTION_REQUEST",
        EventKind::EXECUTION_RESULT => "EXECUTION_RESULT",
        EventKind::CONSEQUENCE => "CONSEQUENCE",
        EventKind::CANCEL => "CANCEL",
        EventKind::CORRECTION => "CORRECTION",
        EventKind::MAINTENANCE => "MAINTENANCE",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Core,
    Task,
    UserView,
    Audit,
}

#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub items: usize,
    pub bytes: usize,
    pub steps: usize,
}

#[derive(Clone, Debug)]
pub struct Budgets {
    pub core: Budget,
    pub task: Budget,
    pub user_view: Budget,
}
impl Default for Budgets {
    fn default() -> Self {
        Self {
            core: Budget {
                items: 16,
                bytes: 16_384,
                steps: 100_000,
            },
            task: Budget {
                items: 32,
                bytes: 32_768,
                steps: 200_000,
            },
            user_view: Budget {
                items: 50,
                bytes: 65_536,
                steps: 300_000,
            },
        }
    }
}
impl Budgets {
    pub fn for_purpose(&self, purpose: Purpose) -> Budget {
        match purpose {
            Purpose::Core => self.core,
            Purpose::Task => self.task,
            Purpose::UserView | Purpose::Audit => self.user_view,
        }
    }
}

/// Explicit, versioned, bounded request. No p, x, m, R, theta or appraisal input.
/// Text is the current observation o (or explicit user query), not an IRL result.
#[derive(Clone, Debug)]
pub struct FactQuery {
    pub history_prefix: String,
    pub version: String,
    pub purpose: Purpose,
    pub text: String,
    pub entities: BTreeSet<String>,
    pub kinds: Vec<EventKind>,
    pub payload_types: BTreeSet<String>,
    pub sources: BTreeSet<String>,
    pub from_ns: Option<i64>,
    pub to_ns: Option<i64>,
    pub budget: Budget,
}
impl FactQuery {
    pub fn new(prefix: &str, purpose: Purpose, text: &str, budgets: &Budgets) -> Self {
        Self {
            history_prefix: prefix.into(),
            version: QUERY_VERSION.into(),
            purpose,
            text: text.into(),
            entities: BTreeSet::new(),
            kinds: vec![],
            payload_types: BTreeSet::new(),
            sources: BTreeSet::new(),
            from_ns: None,
            to_ns: None,
            budget: budgets.for_purpose(purpose),
        }
    }
    pub(crate) fn filter(&mut self, filter: &QueryFilter) -> Result<(), String> {
        self.text = filter.text.clone().unwrap_or_default();
        // Panel date strings use UTC boundaries, independent of host timezone.
        self.from_ns = filter
            .from
            .as_ref()
            .map(|s| parse_time(s, false))
            .transpose()?;
        self.to_ns = filter
            .to
            .as_ref()
            .map(|s| parse_time(s, true))
            .transpose()?;
        self.kinds.clear();
        self.payload_types.clear();
        for s in &filter.kinds {
            if let Some(kind) = [
                EventKind::STARTUP,
                EventKind::OBSERVATION,
                EventKind::ACTION_REQUEST,
                EventKind::EXECUTION_RESULT,
                EventKind::CONSEQUENCE,
                EventKind::CANCEL,
                EventKind::CORRECTION,
                EventKind::MAINTENANCE,
            ]
            .into_iter()
            .find(|k| kind_name(*k) == s.to_uppercase())
            {
                self.kinds.push(kind);
            } else if let Some(tag) = crate::source::payload_tag(s).map_err(|e| e.to_string())? {
                self.payload_types.insert(tag);
            } else {
                return Err("unknown event or payload kind".into());
            }
        }
        Ok(())
    }
}

fn parse_time(s: &str, end_of_day: bool) -> Result<i64, String> {
    if let Ok(n) = s.parse::<i64>() {
        return Ok(n);
    }
    let ns = if s.len() == 10 {
        let format =
            time::format_description::parse("[year]-[month]-[day]").map_err(|e| e.to_string())?;
        let date = time::Date::parse(s, &format).map_err(|e| e.to_string())?;
        let start = date.midnight().assume_utc().unix_timestamp_nanos();
        start
            + if end_of_day {
                86_400_000_000_000 - 1
            } else {
                0
            }
    } else {
        time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
            .map_err(|e| e.to_string())?
            .unix_timestamp_nanos()
    };
    i64::try_from(ns).map_err(|_| "time outside signed nanosecond range".into())
}

/// Issued by the trusted caller's permission layer. Fail closed by default.
/// Restrictions apply before ranking in all four channels.
#[derive(Clone, Debug, Default)]
pub struct Access {
    pub allowed: bool,
    pub audit: bool,
    pub sources: BTreeSet<String>,
}
