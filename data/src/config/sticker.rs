use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Sticker {
    pub enabled: bool,
    pub max_size_px: u32,
    pub packs: Vec<PackEntry>,
}

impl Default for Sticker {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_px: 200,
            packs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackEntry {
    pub url: Url,
}
