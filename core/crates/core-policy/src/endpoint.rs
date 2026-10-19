//! Effect endpoints (O 3.1). Ordered by their frozen IDL code so that every
//! canonical ordering in this crate is stable.

use core_types::generated::EffectEndpoint;

/// O 3.1: F_event, J_direct and G_meta are effect endpoints; F_time is the
/// projection through which θ's natural dynamics are read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endpoint {
    FEvent,
    FTime,
    JDirect,
    GMeta,
}

impl Endpoint {
    /// Frozen IDL enum code (base.proto `EffectEndpoint`).
    pub fn code(self) -> u32 {
        match self {
            Self::FEvent => 1,
            Self::FTime => 2,
            Self::JDirect => 3,
            Self::GMeta => 4,
        }
    }

    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            1 => Some(Self::FEvent),
            2 => Some(Self::FTime),
            3 => Some(Self::JDirect),
            4 => Some(Self::GMeta),
            _ => None,
        }
    }
}

impl From<EffectEndpoint> for Endpoint {
    fn from(value: EffectEndpoint) -> Self {
        match value {
            EffectEndpoint::F_EVENT => Self::FEvent,
            EffectEndpoint::F_TIME => Self::FTime,
            EffectEndpoint::J_DIRECT => Self::JDirect,
            EffectEndpoint::G_META => Self::GMeta,
        }
    }
}

impl From<Endpoint> for EffectEndpoint {
    fn from(value: Endpoint) -> Self {
        match value {
            Endpoint::FEvent => Self::F_EVENT,
            Endpoint::FTime => Self::F_TIME,
            Endpoint::JDirect => Self::J_DIRECT,
            Endpoint::GMeta => Self::G_META,
        }
    }
}
