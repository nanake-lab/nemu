//! Narrow, executable certificate verifier. No world labels, appraisal, x or
//! text enter the decision. Novelty means first recorded verified answer for
//! a canonical arithmetic question, NOT meaningfulness or independence for G.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const VERSION: &str = "euclidean-division/v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub dividend: u64,
    pub divisor: u64,
}

impl Question {
    pub fn key(&self) -> String {
        format!("{VERSION}:{}:{}", self.dividend, self.divisor)
    }
    pub fn digest(&self) -> String {
        Sha256::digest(self.key().as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub action_id: String,
    pub question: Question,
    pub issued_round: u64,
    pub issued_ns: u64,
    pub deadline_ns: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub action_id: String,
    pub question_digest: String,
    pub quotient: u64,
    pub remainder: u64,
    pub verifier_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verdict {
    pub reason: String,
    pub key: String,
    pub known_before: bool,
    pub release: bool,
    pub pulse_allowed: bool,
}

/// Explicit Q-like probe state, saved in evidence and rebuilt by receipt replay.
/// Production integration would require transactionally derived H/Q state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verifier {
    pub version: String,
    pub known: BTreeSet<String>,
    pub completed: BTreeSet<String>,
}

impl Verifier {
    pub fn new(initial: &[Question]) -> Self {
        Self {
            version: VERSION.into(),
            known: initial.iter().map(Question::key).collect(),
            completed: BTreeSet::new(),
        }
    }

    pub fn accept(&mut self, request: &Request, receipt: &Receipt, now_ns: u64) -> Verdict {
        let key = request.question.key();
        let known_before = self.known.contains(&key);
        let mut out = Verdict {
            reason: "unverified".into(),
            key: key.clone(),
            known_before,
            release: false,
            pulse_allowed: false,
        };
        // No receipt can consume another action or extend its original deadline.
        if receipt.action_id != request.action_id
            || receipt.question_digest != request.question.digest()
        {
            out.reason = "binding_mismatch".into();
            return out;
        }
        if now_ns < request.issued_ns || request.deadline_ns < request.issued_ns {
            out.reason = "invalid_time".into();
            return out;
        }
        if !self.completed.insert(request.action_id.clone()) {
            out.reason = "duplicate_completion".into();
            return out;
        }
        if now_ns > request.deadline_ns {
            out.reason = "expired_at_commit".into();
            return out;
        }
        // Unverified/unsupported material retains the ordinary discounted F path.
        // This gate does not stand in for the existing IRL/A/M input governance.
        out.pulse_allowed = true;
        if receipt.verifier_version != self.version {
            out.reason = "unsupported_verifier".into();
            return out;
        }
        let q = &request.question;
        let valid = q.divisor > 0
            && receipt.remainder < q.divisor
            && u128::from(receipt.quotient) * u128::from(q.divisor) + u128::from(receipt.remainder)
                == u128::from(q.dividend);
        if !valid {
            out.reason = "invalid_certificate".into();
            return out;
        }
        out.release = self.known.insert(key);
        out.reason = if out.release {
            "verified_first_result"
        } else {
            "known_question"
        }
        .into();
        out
    }
}
