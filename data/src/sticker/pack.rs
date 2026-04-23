use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

use super::{PackId, StickerId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub cover: Option<String>,
    pub stickers: Vec<StickerDef>,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StickerDef {
    pub id: String,
    pub file: String,
    #[serde(default)]
    pub emoji: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Pack {
    pub id: PackId,
    pub base_url: Url,
    pub manifest: PackManifest,
    /// Local path to the pack's cover image, populated by the cache layer
    /// after the registry fetches pack metadata. `None` if the manifest has
    /// no cover, or if the fetch failed.
    pub cover_path: Option<PathBuf>,
    /// Local path for each sticker image, keyed by the sticker id string
    /// from the manifest. Entries are only present for stickers whose
    /// images were successfully cached.
    pub sticker_paths: BTreeMap<String, PathBuf>,
    /// User-supplied display name override from `config.toml`. When set,
    /// `display_name()` returns this instead of `manifest.name`. Useful for
    /// disambiguating packs that share a manifest name.
    pub label: Option<String>,
}

impl Pack {
    pub fn new(id: PackId, base_url: Url, manifest: PackManifest) -> Self {
        Self {
            id,
            base_url,
            manifest,
            cover_path: None,
            sticker_paths: BTreeMap::new(),
            label: None,
        }
    }

    pub fn find(&self, sticker_id: &StickerId) -> Option<&StickerDef> {
        self.manifest
            .stickers
            .iter()
            .find(|s| s.id == sticker_id.as_str())
    }

    pub fn sticker_url(&self, sticker_id: &StickerId) -> Option<Url> {
        self.base_url.join(&self.find(sticker_id)?.file).ok()
    }

    pub fn sticker_path(&self, sticker_id: &StickerId) -> Option<&PathBuf> {
        self.sticker_paths.get(sticker_id.as_str())
    }

    pub fn cover_url(&self) -> Option<Url> {
        self.base_url.join(self.manifest.cover.as_deref()?).ok()
    }

    /// User's label if they've set one, otherwise the pack.json `name`.
    /// This is what every UI surface should display.
    pub fn display_name(&self) -> &str {
        self.label
            .as_deref()
            .unwrap_or(self.manifest.name.as_str())
    }
}
