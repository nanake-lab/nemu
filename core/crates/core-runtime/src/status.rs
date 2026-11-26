use core_types::generated::{
    Autonomy, DegradedStatus, FailureCode, FailureReason, InitialRuntimeRoot, Lineage, RunStateV2,
    RunStatus, SpeechMode, TerminalLineage,
};

/// Capability intersection. No worker recovery clears another fault.
#[derive(Clone, Debug)]
pub struct Status {
    value: DegradedStatus,
    autonomous: bool,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            value: DegradedStatus {
                schema_version: 1,
                states: vec![],
                reasons: vec![],
            },
            autonomous: true,
        }
    }
}
impl Status {
    pub(crate) fn unmodulated(lineage: Lineage, q: Option<&InitialRuntimeRoot>) -> Self {
        let mut status = Self::default();
        if let Some(q) = q {
            for (stage, state) in [
                (&q.irl, RunStatus::IRL_DEGRADED),
                (&q.appraiser, RunStatus::A_DEGRADED),
                (&q.matcher, RunStatus::M_DEGRADED),
            ] {
                if stage.is_none() {
                    status.add(state, FailureCode::DEPENDENCY_FAULT);
                }
            }
        }
        status.add(
            RunStatus::CORE_UNMODULATED,
            FailureCode::MODULATION_DISABLED,
        );
        if lineage == Lineage::REHEARSAL {
            status.add(RunStatus::NO_GROWTH, FailureCode::GROWTH_OFF_REHEARSAL);
        }
        status
    }

    pub fn add_state(&mut self, state: RunStatus) {
        if !self.value.states.contains(&state) {
            self.value.states.push(state);
        }
        self.value.states.sort_by_key(|v| u32::from(*v));
        if matches!(
            state,
            RunStatus::CORE_UNMODULATED
                | RunStatus::MODULATION_NOT_ENABLED
                | RunStatus::STORAGE_RECOVERY_FAULT
        ) {
            self.autonomous = false;
        }
    }
    pub fn add(&mut self, state: RunStatus, reason: FailureCode) {
        self.add_state(state);
        if !self.value.reasons.iter().any(|r| r.code == reason) {
            self.value.reasons.push(FailureReason {
                schema_version: 1,
                code: reason,
                detail_ref: None,
            });
        }
        self.value.states.sort_by_key(|v| u32::from(*v));
        self.value.reasons.sort_by_key(|r| u32::from(r.code));
    }
    pub fn pause_autonomy(&mut self) {
        self.autonomous = false;
    }
    pub fn autonomy(&self) -> bool {
        self.autonomous
    }
    pub fn value(&self) -> DegradedStatus {
        let mut value = self.value.clone();
        if value.states.is_empty() {
            value.states.push(RunStatus::NORMAL);
        }
        value
    }
    /// EXPERIMENT has no terminal wire representation. Never relabel it LIFE.
    pub fn terminal(
        &self,
        lineage: Lineage,
        mode: SpeechMode,
        strict: bool,
        born_at: Option<String>,
    ) -> Option<RunStateV2> {
        let lineage = match lineage {
            Lineage::REHEARSAL => TerminalLineage::TERMINAL_REHEARSAL,
            Lineage::LIFE => TerminalLineage::TERMINAL_LIFE,
            Lineage::EXPERIMENT => return None,
        };
        let value = self.value();
        Some(RunStateV2 {
            schema_version: 1,
            states: value.states,
            speech_mode: mode,
            strict,
            born_at,
            autonomy: if self.autonomous {
                Autonomy::ACTIVE_AUTONOMY
            } else {
                Autonomy::PAUSED
            },
            reasons: value
                .reasons
                .iter()
                .map(|r| format!("{:?}", r.code))
                .collect(),
            lineage,
        })
    }
}
