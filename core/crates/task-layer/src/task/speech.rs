//! D3, D6, D7 and D9. Source/semantic governance is supplied by the trusted
//! host. A label chosen by the speaking model cannot establish a past fact.
use core_types::generated::{QuoteLayer, RecordVersion};
use serde::Deserialize;
use std::{collections::BTreeMap, ops::Range};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MissingHint {
    None,
    #[default]
    PerEvent,
    General,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelfQuote {
    #[default]
    Loose,
    Strict,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpeechSettings {
    pub missing: MissingHint,
    pub self_quote: SelfQuote,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    PastEvent,
    PastSelf,
    Now,
    Other,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    #[serde(rename = "type")]
    pub kind: Kind,
    pub text: String,
    pub cites: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Outline {
    points: Vec<Point>,
}

/// Each span refers to visible material in the actual packed request. A trusted
/// producer verifies identity evidence and external roots before admission.
pub struct VisibleQuote<'a> {
    pub principal: &'a str,
    pub text: &'a str,
}
pub fn visible_quote<'a>(
    version: &RecordVersion,
    layer: &'a QuoteLayer,
    body: &'a str,
    delivered: Range<usize>,
) -> Option<VisibleQuote<'a>> {
    let start = usize::try_from(layer.visible.start.0).ok()?;
    let end = usize::try_from(layer.visible.end.0).ok()?;
    if version != &layer.source || start >= end || start < delivered.start || end > delivered.end {
        return None;
    }
    Some(VisibleQuote {
        principal: &layer.principal_id,
        text: body.get(start..end)?,
    })
}

/// The review checks the meaning of each claim against exactly the cited visible
/// words (including loose paraphrases), and classification even for uncited now/other.
/// Rejecting all points yields no speech. Never infer absence of a historical record.
pub fn join_checked(
    raw: &str,
    settings: SpeechSettings,
    self_id: &str,
    quotes: &BTreeMap<String, Vec<VisibleQuote<'_>>>,
    mut supported: impl FnMut(&Point, &[&VisibleQuote<'_>]) -> bool,
) -> Result<String, String> {
    let plan: Outline = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let mut approved = Vec::new();
    for point in plan.points {
        let past = matches!(point.kind, Kind::PastEvent | Kind::PastSelf);
        if point.text.trim().is_empty() || past == point.cites.is_empty() {
            continue;
        }
        let Some(groups) = point
            .cites
            .iter()
            .map(|id| quotes.get(id))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        if groups.iter().any(|g| g.is_empty()) {
            continue;
        }
        let evidence: Vec<_> = groups.into_iter().flatten().collect();
        if point.kind == Kind::PastSelf
            && !evidence.iter().any(|q| {
                q.principal == self_id
                    && (settings.self_quote == SelfQuote::Loose || q.text == point.text)
            })
        {
            continue;
        }
        if supported(&point, &evidence) {
            approved.push(point.text)
        }
    }
    Ok(approved.concat())
}
