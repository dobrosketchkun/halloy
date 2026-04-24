use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Sticker {
    pub enabled: bool,
    pub max_size_px: u32,
    /// Reject `pack.json` responses larger than this many bytes. Defends
    /// against a malicious repo serving a huge manifest that would blow
    /// up JSON parsing memory.
    pub max_manifest_bytes: usize,
    /// Reject individual sticker images larger than this many bytes during
    /// fetch. Failed stickers are silently skipped (not shown in the picker).
    pub max_image_bytes: usize,
    /// Keep at most this many stickers per pack. Extra stickers in the
    /// manifest are silently truncated.
    pub max_stickers_per_pack: usize,
    pub packs: Vec<PackEntry>,
}

impl Default for Sticker {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_px: 200,
            max_manifest_bytes: 64 * 1024,
            max_image_bytes: 512 * 1024,
            max_stickers_per_pack: 120,
            packs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackEntry {
    pub url: Url,
    #[serde(default)]
    pub label: Option<String>,
}
