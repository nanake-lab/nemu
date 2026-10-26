//! Pure confirmation policy and growth-command constraints (W22).
//!
//! These predicates neither authenticate callers nor issue execution credentials.
//! Inputs must come from an authenticated runtime adapter, never from payload
//! identity claims. No permission memory is stored here (I18).
//! See tasks/requests/P5-05-permission-integration.md for the blocked integration.

pub mod growth;
pub mod tier;
