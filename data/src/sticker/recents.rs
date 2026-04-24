use std::collections::VecDeque;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{PackId, StickerId, StickerRef};
use crate::environment;

/// Lives alongside `drafts.json`, `known_filehosts.json`, etc. in halloy's
/// data dir. Not in config.toml — runtime state shouldn't contaminate the
/// user's hand-edited config.
fn recents_path() -> PathBuf {
    environment::data_dir().join("sticker_recents.json")
}

#[derive(Serialize, Deserialize)]
struct Entry {
    pack: String,
    sticker: String,
}

/// Read the recents list from disk. Returns an empty list if the file
/// doesn't exist, isn't valid JSON, or contains entries whose pack/sticker
/// ids are now malformed — the user simply starts with empty recents.
/// Stale entries (packs the user has since removed) are kept on disk and
/// filtered at render time, so re-adding a pack resurrects its recents.
pub fn load_sync() -> VecDeque<StickerRef> {
    let path = recents_path();
    let Ok(bytes) = std::fs::read(&path) else {
        return VecDeque::new();
    };
    let Ok(entries) = serde_json::from_slice::<Vec<Entry>>(&bytes) else {
        return VecDeque::new();
    };
    entries
        .into_iter()
        .filter_map(|e| {
            let pack = PackId::new(e.pack)?;
            let sticker = StickerId::new(e.sticker)?;
            Some(StickerRef { pack, sticker })
        })
        .collect()
}

/// Blocking write. ~1 KB file; negligible latency on the main thread.
pub fn save_sync(refs: &[StickerRef]) -> std::io::Result<()> {
    let path = recents_path();
    let entries: Vec<Entry> = refs
        .iter()
        .map(|r| Entry {
            pack: r.pack.as_str().to_owned(),
            sticker: r.sticker.as_str().to_owned(),
        })
        .collect();
    let bytes = serde_json::to_vec_pretty(&entries).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}
