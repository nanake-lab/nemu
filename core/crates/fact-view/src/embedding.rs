use crate::{Error, Result};

/// In-process only: no worker, network service or handwritten IPC messages.
pub trait Embedder {
    fn version(&self) -> &str;
    fn passages(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn query(&mut self, text: &str) -> Result<Vec<f32>>;
}
pub(crate) fn vector_bytes(vector: &[f32]) -> Result<Vec<u8>> {
    if vector.is_empty()
        || vector.len() > 4096
        || vector.iter().any(|x| !x.is_finite())
        || vector.iter().all(|x| *x == 0.0)
    {
        return Err(Error::Invalid("invalid embedding".into()));
    }
    Ok(vector.iter().flat_map(|v| v.to_le_bytes()).collect())
}

#[cfg(feature = "cpu-embedding")]
mod cpu;
#[cfg(feature = "cpu-embedding")]
pub use cpu::CpuEmbedder;
