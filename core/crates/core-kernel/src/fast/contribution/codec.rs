use super::*;
use core_types::{
    WireMessage,
    contract::{Limits, Schema},
    generated::*,
};

// Contract boundary with the same caller limits for structural and semantic
// checks (P5-68). Decode to the schema value without a second runtime JSON dep.
pub(super) fn check<T: WireMessage>(message: &T, limits: Limits) -> Result<(), FastError> {
    let bytes = message.canonical_bytes(limits).map_err(FastError::Codec)?;
    let value = Schema::builtin()
        .and_then(|s| s.decode_canonical(T::NAME, &bytes, limits, 0))
        .map_err(FastError::Codec)?;
    core_types::semantic::check_law_state_message(T::NAME, &value, limits).map_err(FastError::Codec)
}

impl State {
    pub fn from_snapshot(
        snapshot: FastStateSnapshotV3,
        law: &ObjectRef,
        limits: Limits,
    ) -> Result<Self, FastError> {
        check(&snapshot, limits)?;
        if snapshot.representation_version != REPRESENTATION_VERSION {
            return Err(FastError::RepresentationVersion(
                snapshot.representation_version,
            ));
        }
        if &snapshot.law_version != law {
            return Err(FastError::OutOfRange("law binding"));
        }
        let snapshot = FastStateSnapshotV3::from_canonical(
            &snapshot.canonical_bytes(limits).map_err(FastError::Codec)?,
            limits,
        )
        .map_err(FastError::Codec)?;
        Ok(Self { snapshot })
    }

    pub fn from_canonical(
        bytes: &[u8],
        law: &ObjectRef,
        limits: Limits,
    ) -> Result<Self, FastError> {
        Self::from_snapshot(
            FastStateSnapshotV3::from_canonical(bytes, limits).map_err(FastError::Codec)?,
            law,
            limits,
        )
    }

    pub fn canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>, FastError> {
        self.snapshot
            .canonical_bytes(limits)
            .map_err(FastError::Codec)
    }

    pub fn visible(&self) -> super::super::FastState {
        let n = &self.snapshot.nature;
        super::super::FastState {
            nature: [n.connection, n.empathy, n.curiosity, n.achievement],
            learned: self
                .snapshot
                .learned
                .iter()
                .map(|l| super::super::LearnedValue {
                    tendency_id: l.tendency_id.clone(),
                    value: l.value,
                })
                .collect(),
        }
    }

    pub(super) fn set_visible(&mut self, x: &super::super::FastState) {
        self.snapshot.nature = NatureVector {
            schema_version: 1,
            connection: x.nature[0],
            empathy: x.nature[1],
            curiosity: x.nature[2],
            achievement: x.nature[3],
        };
        self.snapshot.learned = x
            .learned
            .iter()
            .map(|l| LearnedComponent {
                schema_version: 1,
                tendency_id: l.tendency_id.clone(),
                value: l.value,
            })
            .collect();
    }
}

pub(super) fn vector(d: &Drive) -> LawStateVector {
    LawStateVector {
        schema_version: 1,
        nature: NatureReal {
            schema_version: 1,
            connection: d.nature[0],
            empathy: d.nature[1],
            curiosity: d.nature[2],
            achievement: d.nature[3],
        },
        learned: d
            .learned
            .iter()
            .map(|l| LawLearnedCoordinate {
                schema_version: 1,
                tendency_id: l.tendency_id.clone(),
                value: l.value,
            })
            .collect(),
    }
}

pub(super) fn values(v: &LawStateVector) -> Vec<f64> {
    let n = &v.nature;
    [n.connection, n.empathy, n.curiosity, n.achievement]
        .into_iter()
        .chain(v.learned.iter().map(|l| l.value))
        .collect()
}
pub(super) fn set_values(v: &mut LawStateVector, values: &[f64]) {
    v.nature.connection = values[0];
    v.nature.empathy = values[1];
    v.nature.curiosity = values[2];
    v.nature.achievement = values[3];
    for (l, value) in v.learned.iter_mut().zip(&values[4..]) {
        l.value = *value;
    }
}

/// ADR 0035 §§4.3, 4.7: drop departed coordinates, zero newly active ones.
pub(super) fn align(v: &LawStateVector, learned: &[LearnedComponent]) -> LawStateVector {
    LawStateVector {
        schema_version: 1,
        nature: v.nature.clone(),
        learned: learned
            .iter()
            .map(|l| LawLearnedCoordinate {
                schema_version: 1,
                tendency_id: l.tendency_id.clone(),
                value: v
                    .learned
                    .iter()
                    .find(|c| c.tendency_id == l.tendency_id)
                    .map_or(0.0, |c| c.value),
            })
            .collect(),
    }
}

pub(super) fn sort_slots(state: &mut State, limits: Limits) -> Result<(), FastError> {
    let mut keyed = state
        .snapshot
        .slots
        .drain(..)
        .map(|s| {
            Ok((
                s.basis.canonical_bytes(limits).map_err(FastError::Codec)?,
                s,
            ))
        })
        .collect::<Result<Vec<_>, FastError>>()?;
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    state.snapshot.slots = keyed.into_iter().map(|(_, s)| s).collect();
    Ok(())
}
