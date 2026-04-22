pub mod fetch;
pub mod pack;
pub mod registry;
pub mod wire;

pub use pack::{Pack, PackManifest, StickerDef};
pub use registry::Registry;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid JSON manifest: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid pack URL: {0}")]
    BadUrl(#[from] url::ParseError),
    #[error("invalid pack id {0:?}")]
    InvalidPackId(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackId(String);

impl PackId {
    pub fn new(s: impl Into<String>) -> Option<Self> {
        let s: String = s.into();
        if s.is_empty() || s.chars().any(|c| c == '/' || c.is_whitespace()) {
            None
        } else {
            Some(Self(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StickerId(String);

impl StickerId {
    pub fn new(s: impl Into<String>) -> Option<Self> {
        let s: String = s.into();
        if s.is_empty() || s.chars().any(|c| c == '/' || c.is_whitespace()) {
            None
        } else {
            Some(Self(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StickerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StickerRef {
    pub pack: PackId,
    pub sticker: StickerId,
}

impl std::fmt::Display for StickerRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.pack, self.sticker)
    }
}
