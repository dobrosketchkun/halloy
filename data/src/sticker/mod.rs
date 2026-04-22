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

// ============================================================================
// Shared registry singleton.
//
// The sticker registry is genuinely app-global state: one client, one loaded
// set of packs, loaded at startup and refreshed on config reload. Threading
// a `&Registry` through the 10 layers between the Halloy struct and the
// command parser would add ~30 signature changes across many files. A
// `OnceLock<Arc<RwLock<Registry>>>` gets us the same correctness with local
// changes. Access is always synchronous and read-only in hot paths, so
// the RwLock contention is a non-issue.
// ============================================================================

use std::sync::{Arc, OnceLock, RwLock};

static SHARED: OnceLock<Arc<RwLock<Registry>>> = OnceLock::new();

fn shared_cell() -> &'static Arc<RwLock<Registry>> {
    SHARED.get_or_init(|| Arc::new(RwLock::new(Registry::default())))
}

pub fn with_shared<F, R>(f: F) -> R
where
    F: FnOnce(&Registry) -> R,
{
    let guard = shared_cell()
        .read()
        .expect("sticker registry lock poisoned");
    f(&guard)
}

pub fn replace_shared(registry: Registry) {
    if let Ok(mut guard) = shared_cell().write() {
        *guard = registry;
    }
}

pub fn resolve_url(
    pack_id: &PackId,
    sticker_id: &StickerId,
) -> Option<url::Url> {
    with_shared(|reg| {
        reg.get(pack_id).and_then(|p| p.sticker_url(sticker_id))
    })
}
