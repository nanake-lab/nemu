//! V2-78/V2-95: a pure projection of recorded H times, never an OS uptime read.
use super::{Fault, Result};

/// A complete, authenticated stop interval from H, in the event-time domain.
/// Authentication and completeness belong to the caller, not this local DTO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Downtime {
    pub start_ns: u64,
    pub end_ns: u64,
}

/// Temporary in-memory projection; not a frozen storage/wire contract (V2-98).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunningTime {
    pub(super) origin_ns: u64,
    pub(super) event_ns: u64,
    pub(super) stopped_ns: u64,
    pub(super) running_ns: u64,
}

impl RunningTime {
    /// Replay the complete ordered list of stops since this lineage's time origin.
    /// Stops must be closed, nonempty, nonoverlapping and end by this event.
    pub fn from_history(origin_ns: u64, event_ns: u64, stops: &[Downtime]) -> Result<Self> {
        let elapsed = event_ns.checked_sub(origin_ns).ok_or(Fault::EventOrder)?;
        let mut end = origin_ns;
        let mut stopped = 0_u64;
        for stop in stops {
            if stop.start_ns < end || stop.end_ns <= stop.start_ns || stop.end_ns > event_ns {
                return Err(Fault::InvalidRunningTime);
            }
            stopped = stopped
                .checked_add(stop.end_ns - stop.start_ns)
                .ok_or(Fault::InvalidRunningTime)?;
            end = stop.end_ns;
        }
        Ok(Self {
            origin_ns,
            event_ns,
            stopped_ns: stopped,
            running_ns: elapsed
                .checked_sub(stopped)
                .ok_or(Fault::InvalidRunningTime)?,
        })
    }

    pub fn running_ns(&self) -> u64 {
        self.running_ns
    }

    pub(super) fn validate_after(&self, old: Option<&Self>, event_ns: u64) -> Result<()> {
        if self.event_ns != event_ns {
            return Err(Fault::InvalidRunningTime);
        }
        if let Some(old) = old {
            let wall = self
                .event_ns
                .checked_sub(old.event_ns)
                .ok_or(Fault::EventOrder)?;
            let stopped = self
                .stopped_ns
                .checked_sub(old.stopped_ns)
                .ok_or(Fault::InvalidRunningTime)?;
            if self.origin_ns != old.origin_ns || stopped > wall || self.running_ns < old.running_ns
            {
                return Err(Fault::InvalidRunningTime);
            }
        }
        Ok(())
    }
}
