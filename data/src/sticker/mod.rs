pub mod cache;
pub mod fetch;
pub mod pack;
pub mod persist;
pub mod recents;
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

use std::collections::VecDeque;
use std::sync::{Arc, OnceLock, RwLock};

static SHARED: OnceLock<Arc<RwLock<Registry>>> = OnceLock::new();
static SHARED_RECENTS: OnceLock<Arc<RwLock<VecDeque<StickerRef>>>> =
    OnceLock::new();

const MAX_RECENTS: usize = 24;

fn shared_cell() -> &'static Arc<RwLock<Registry>> {
    SHARED.get_or_init(|| Arc::new(RwLock::new(Registry::default())))
}

fn recents_cell() -> &'static Arc<RwLock<VecDeque<StickerRef>>> {
    SHARED_RECENTS
        .get_or_init(|| Arc::new(RwLock::new(VecDeque::new())))
}

/// Record a sticker send in the MRU recents list and persist to disk.
/// Moves the entry to the front if already present (so repeatedly sending
/// the same sticker doesn't fill up recents with duplicates). Capped at
/// `MAX_RECENTS`.
pub fn push_recent(pack_id: PackId, sticker_id: StickerId) {
    let snapshot: Vec<StickerRef> = {
        let Ok(mut guard) = recents_cell().write() else {
            return;
        };
        let new_ref = StickerRef {
            pack: pack_id,
            sticker: sticker_id,
        };
        guard.retain(|r| r != &new_ref);
        guard.push_front(new_ref);
        while guard.len() > MAX_RECENTS {
            guard.pop_back();
        }
        guard.iter().cloned().collect()
    };
    if let Err(err) = recents::save_sync(&snapshot) {
        log::warn!("failed to save sticker recents: {err}");
    }
}

/// Install a previously-loaded recents snapshot into the shared cell.
/// Called once at startup after reading `sticker_recents.json`.
pub fn replace_shared_recents(new: VecDeque<StickerRef>) {
    if let Ok(mut guard) = recents_cell().write() {
        *guard = new;
    }
}

/// Snapshot of recents in MRU order. Most-recently-sent first.
pub fn recents() -> Vec<StickerRef> {
    recents_cell()
        .read()
        .map(|g| g.iter().cloned().collect())
        .unwrap_or_default()
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

pub fn with_shared_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Registry) -> R,
{
    let mut guard = shared_cell()
        .write()
        .expect("sticker registry lock poisoned");
    f(&mut guard)
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

pub fn resolve_path(
    pack_id: &PackId,
    sticker_id: &StickerId,
) -> Option<std::path::PathBuf> {
    with_shared(|reg| {
        reg.get(pack_id)
            .and_then(|p| p.sticker_path(sticker_id))
            .cloned()
    })
}

/// Reverse lookup: find the pack that owns a given sticker image URL.
/// Used when a user clicks a sticker in chat — we have the URL from the
/// message preview, and want to open the pack-info modal for its pack.
pub fn pack_for_url(url: &url::Url) -> Option<PackId> {
    with_shared(|reg| {
        for pack in reg.iter() {
            for s in &pack.manifest.stickers {
                if let Ok(candidate) = pack.base_url.join(&s.file)
                    && &candidate == url
                {
                    return Some(pack.id.clone());
                }
            }
        }
        None
    })
}

/// If `pack.id` already exists in `existing`, rewrite it to a locally-
/// unique form by appending a short URL-derived hash. Both paths that
/// insert packs (startup load from config and manual Add via the manager
/// modal) go through this helper so behaviour is consistent.
///
/// Two unrelated packs can legitimately share the same `"id"` string in
/// their `pack.json` (different authors using the same shortname); this
/// keeps both in the registry under distinct keys. Cross-user wire-tag
/// identity is best-effort — URL-based lookup (`pack_for_url`) is the
/// authoritative resolution path for incoming sticker messages.
pub(crate) fn disambiguate_local_id(
    pack: &mut Pack,
    url: &url::Url,
    existing: &Registry,
) {
    if existing.get(&pack.id).is_none() {
        return;
    }
    let hash = seahash::hash(url.as_str().as_bytes()) & 0xFF_FFFF;
    if let Some(new_id) =
        PackId::new(format!("{}_{hash:06x}", pack.id))
    {
        log::info!(
            "pack id \"{}\" already loaded; using local id \"{}\" for {}",
            pack.id,
            new_id,
            url
        );
        pack.id = new_id;
    }
}

/// Orchestrates: fetch a pack.json + cache images + insert into shared
/// registry + persist to config.toml. Any failure bubbles up as a
/// human-readable error string for display in the manager modal.
pub async fn add_and_persist(url: url::Url) -> Result<PackId, String> {
    let mut pack = fetch::fetch_pack(url.clone())
        .await
        .map_err(|e| format!("failed to fetch pack: {e}"))?;

    // Disambiguate BEFORE caching images — image paths live under the
    // pack id's folder, so we want the final id settled first.
    with_shared(|r| disambiguate_local_id(&mut pack, &url, r));

    let cached = registry::cache_pack_images(pack).await;
    let id = cached.id.clone();
    log::info!("adding sticker pack {id}");
    with_shared_mut(|r| r.insert(cached));
    persist::persist_registry()
        .await
        .map_err(|e| format!("failed to save config: {e}"))?;
    Ok(id)
}

/// Remove a pack from the shared registry and persist the updated list to
/// config.toml. No-op (still returns Ok) if the pack isn't loaded.
pub async fn remove_and_persist(pack_id: PackId) -> Result<(), String> {
    let removed = with_shared_mut(|r| r.remove(&pack_id)).is_some();
    if !removed {
        return Ok(());
    }
    persist::persist_registry()
        .await
        .map_err(|e| format!("failed to save config: {e}"))
}

/// Swap the pack one slot up (toward the start of the list) and persist.
/// No-op if the pack doesn't exist or is already first.
pub async fn move_up_and_persist(pack_id: PackId) -> Result<(), String> {
    let moved = with_shared_mut(|r| r.move_up(&pack_id));
    if !moved {
        return Ok(());
    }
    persist::persist_registry()
        .await
        .map_err(|e| format!("failed to save config: {e}"))
}

/// Swap the pack one slot down (toward the end) and persist. No-op if the
/// pack doesn't exist or is already last.
pub async fn move_down_and_persist(pack_id: PackId) -> Result<(), String> {
    let moved = with_shared_mut(|r| r.move_down(&pack_id));
    if !moved {
        return Ok(());
    }
    persist::persist_registry()
        .await
        .map_err(|e| format!("failed to save config: {e}"))
}

/// Set a user-supplied display label for the pack and persist. Passing an
/// empty string or all-whitespace clears the label (reverts to
/// `manifest.name`).
pub async fn set_label_and_persist(
    pack_id: PackId,
    label: String,
) -> Result<(), String> {
    let trimmed = label.trim();
    let new_label = (!trimmed.is_empty()).then(|| trimmed.to_owned());
    let changed = with_shared_mut(|r| {
        if let Some(pack) = r.get_mut(&pack_id) {
            pack.label = new_label;
            true
        } else {
            false
        }
    });
    if !changed {
        return Ok(());
    }
    persist::persist_registry()
        .await
        .map_err(|e| format!("failed to save config: {e}"))
}
