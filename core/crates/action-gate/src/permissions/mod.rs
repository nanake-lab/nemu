//! Pure confirmation policy and growth-command constraints (W22).
//!
//! These predicates neither authenticate callers nor issue execution credentials.
//! Inputs must come from an authenticated runtime adapter, never from payload
//! identity claims. No permission memory is stored here (I18).
//! Identity, credentials and runtime integration are handed off to P5-40.

pub mod approval;
pub mod growth;
pub mod tier;
