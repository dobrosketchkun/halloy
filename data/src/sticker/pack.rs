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
}

impl Pack {
    pub fn new(id: PackId, base_url: Url, manifest: PackManifest) -> Self {
        Self {
            id,
            base_url,
            manifest,
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

    pub fn cover_url(&self) -> Option<Url> {
        self.base_url.join(self.manifest.cover.as_deref()?).ok()
    }
}
