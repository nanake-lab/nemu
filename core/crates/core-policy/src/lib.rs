#![forbid(unsafe_code)]
//! Echo Core policy crate.
//!
//! The v1.0 scope DSL, its field catalog, endpoint whitelist and overlap
//! check lived here (P5-09). Echo Core v2.0 §15 removed them: an effect of a
//! learned tendency applies wherever its situation match μ says it applies,
//! and there is no scope algebra left to interpret (P5-27). The crate is kept
//! as the home of future pure policy rules so that directory ownership in
//! AGENTS.md stays stable; it currently exports nothing.
