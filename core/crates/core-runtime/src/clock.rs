//! In-process time calculations, not persisted messages. No wall-clock guessing.
//! A caller must obtain checkpoint samples from a future authoritative clock
//! record. There is deliberately no production restart adapter until that IDL
//! and its atomic store publication exist.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sample {
    pub boot: String,
    pub epoch: String,
    pub monotonic_ns: u64,
    pub wall_ns: i64,
    /// A bound supplied by a trusted time provider; None means untrusted.
    pub uncertainty_ns: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub max_interval_ns: u64,
    pub max_wall_skew_ns: u64,
    pub max_uncertainty_ns: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockFault {
    InvalidIdentity,
    Untrusted,
    Backwards,
    Jump,
    Overflow,
    EpochChanged,
}

/// Time is relative to the persisted logical anchor, never the event timestamp.
#[derive(Clone, Debug)]
pub struct Clock {
    origin: Sample,
    origin_process_ns: u64,
    last: Sample,
    bounds: Bounds,
}
#[derive(Debug)]
pub struct ProcessTime {
    process_ns: u64,
    sample: Sample,
}
impl ProcessTime {
    pub fn ns(&self) -> u64 {
        self.process_ns
    }
    pub fn sample(&self) -> &Sample {
        &self.sample
    }
}
pub trait Provider {
    fn sample(&mut self) -> Sample;
}

impl Clock {
    pub fn start(sample: Sample, process_ns: u64, bounds: Bounds) -> Result<Self, ClockFault> {
        checked(&sample, bounds)?;
        if bounds.max_interval_ns == 0 {
            return Err(ClockFault::Jump);
        }
        Ok(Self {
            origin: sample.clone(),
            last: sample,
            origin_process_ns: process_ns,
            bounds,
        })
    }
    /// An explicitly supplied authoritative checkpoint is required. A changed
    /// boot/epoch uses bounded trusted wall time, never monotonic subtraction.
    pub fn restore(
        previous: Sample,
        process_ns: u64,
        now: Sample,
        bounds: Bounds,
    ) -> Result<Self, ClockFault> {
        checked(&previous, bounds)?;
        checked(&now, bounds)?;
        let elapsed = if previous.boot == now.boot && previous.epoch == now.epoch {
            interval(&previous, &now, bounds)?
        } else {
            let wall = wall_interval(&previous, &now)?;
            if wall > bounds.max_interval_ns {
                return Err(ClockFault::Jump);
            }
            wall
        };
        let logical = process_ns
            .checked_add(elapsed)
            .ok_or(ClockFault::Overflow)?;
        Self::start(now, logical, bounds)
    }
    /// Exactly one provider sample per accepted transaction. Filtered ingress
    /// never calls this method. A failed transaction does not publish an anchor.
    pub fn capture(&mut self, source: &mut impl Provider) -> Result<ProcessTime, ClockFault> {
        let sample = source.sample();
        interval(&self.last, &sample, self.bounds)?;
        let elapsed = sample
            .monotonic_ns
            .checked_sub(self.origin.monotonic_ns)
            .ok_or(ClockFault::Backwards)?;
        let process_ns = self
            .origin_process_ns
            .checked_add(elapsed)
            .ok_or(ClockFault::Overflow)?;
        self.last = sample.clone();
        Ok(ProcessTime { process_ns, sample })
    }
    /// Separate persistence-completion observation. It never alters process time
    /// or x. Its durable, atomic representation remains a successor-contract task.
    pub fn completion(
        &mut self,
        time: &ProcessTime,
        source: &mut impl Provider,
    ) -> Result<Sample, ClockFault> {
        let sample = source.sample();
        interval(&self.last, &sample, self.bounds)?;
        interval(time.sample(), &sample, self.bounds)?;
        self.last = sample.clone();
        Ok(sample)
    }
}
fn checked(s: &Sample, bounds: Bounds) -> Result<(), ClockFault> {
    if s.boot.is_empty() || s.epoch.is_empty() {
        return Err(ClockFault::InvalidIdentity);
    }
    if s.uncertainty_ns
        .is_none_or(|u| u > bounds.max_uncertainty_ns)
    {
        return Err(ClockFault::Untrusted);
    }
    Ok(())
}
fn wall_interval(a: &Sample, b: &Sample) -> Result<u64, ClockFault> {
    let delta = i128::from(b.wall_ns) - i128::from(a.wall_ns);
    u64::try_from(delta).map_err(|_| ClockFault::Backwards)
}
fn interval(a: &Sample, b: &Sample, bounds: Bounds) -> Result<u64, ClockFault> {
    checked(a, bounds)?;
    checked(b, bounds)?;
    if a.boot != b.boot || a.epoch != b.epoch {
        return Err(ClockFault::EpochChanged);
    }
    let elapsed = b
        .monotonic_ns
        .checked_sub(a.monotonic_ns)
        .ok_or(ClockFault::Backwards)?;
    let wall = wall_interval(a, b)?;
    let uncertainty = a
        .uncertainty_ns
        .unwrap()
        .checked_add(b.uncertainty_ns.unwrap())
        .and_then(|u| u.checked_add(bounds.max_wall_skew_ns))
        .ok_or(ClockFault::Overflow)?;
    if elapsed > bounds.max_interval_ns || elapsed.abs_diff(wall) > uncertainty {
        return Err(ClockFault::Jump);
    }
    Ok(elapsed)
}
