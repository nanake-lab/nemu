use crate::{Embedder, Error, Result};
use fastembed::{
    InitOptionsUserDefined, Pooling, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
struct Asset {
    path: String,
    sha256: String,
    bytes: u64,
}
#[derive(Deserialize)]
struct Manifest {
    model: String,
    revision: String,
    license: String,
    files: Vec<Asset>,
}

pub struct CpuEmbedder {
    model: TextEmbedding,
    version: String,
}
impl CpuEmbedder {
    /// Offline only. Each file is verified against the repository-pinned
    /// manifest before loading. Default build has no GPU provider features.
    pub fn load(directory: &Path) -> Result<Self> {
        let manifest: Manifest = serde_json::from_str(include_str!("../../models/manifest.json"))?;
        if manifest.license != "MIT" {
            return Err(Error::Invalid("unapproved model license".into()));
        }
        let mut files = BTreeMap::new();
        for asset in manifest.files {
            let bytes = std::fs::read(directory.join(&asset.path))?;
            if bytes.len() as u64 != asset.bytes
                || core_types::scalar::hex(&Sha256::digest(&bytes)) != asset.sha256
            {
                return Err(Error::Invalid(format!("model integrity: {}", asset.path)));
            }
            files.insert(asset.path, bytes);
        }
        let mut take = |name: &str| {
            files
                .remove(name)
                .ok_or_else(|| Error::Invalid(format!("model file absent: {name}")))
        };
        let tokenizer = TokenizerFiles {
            tokenizer_file: take("tokenizer.json")?,
            config_file: take("config.json")?,
            special_tokens_map_file: take("special_tokens_map.json")?,
            tokenizer_config_file: take("tokenizer_config.json")?,
        };
        let model = UserDefinedEmbeddingModel::new(take("model.onnx")?, tokenizer)
            .with_pooling(Pooling::Mean);
        let options = InitOptionsUserDefined::new()
            .with_execution_providers(vec![])
            .with_intra_threads(4)
            .with_max_length(512);
        let model = TextEmbedding::try_new_from_user_defined(model, options)
            .map_err(|e| Error::Invalid(e.to_string()))?;
        Ok(Self {
            model,
            version: format!(
                "{}@{};fp32;mean-l2;512;cpu4",
                manifest.model, manifest.revision
            ),
        })
    }
    fn encode(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let vectors = self
            .model
            .embed(texts, Some(16))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        for vector in &vectors {
            if vector.len() != 384 {
                return Err(Error::Invalid("unexpected E5 dimension".into()));
            }
            super::vector_bytes(vector)?;
        }
        Ok(vectors)
    }
}
impl Embedder for CpuEmbedder {
    fn version(&self) -> &str {
        &self.version
    }
    fn passages(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.encode(texts.iter().map(|s| format!("passage: {s}")).collect())
    }
    fn query(&mut self, text: &str) -> Result<Vec<f32>> {
        self.encode(vec![format!("query: {text}")])?
            .pop()
            .ok_or(Error::Invalid("no query vector".into()))
    }
}
