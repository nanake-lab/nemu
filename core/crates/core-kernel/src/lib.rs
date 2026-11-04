#![forbid(unsafe_code)]
//! Echo Core kernel: pure transitions only. No storage, network, clock or
//! model access.

pub mod dynamics;
pub mod effects;
pub mod fast;
pub mod law;
pub mod modulation;

// P5-24/P5-26 条件性交付：只进测试构建，不暴露给生产。
#[cfg(test)]
pub mod evidence;
#[cfg(test)]
pub mod recompute;
