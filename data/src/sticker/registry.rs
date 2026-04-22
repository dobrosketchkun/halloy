use std::collections::BTreeMap;
use std::path::PathBuf;

use indexmap::IndexMap;

use super::{Pack, PackId};
use crate::config;

#[derive(Debug, Default, Clone)]
pub struct Registry {
    packs: IndexMap<PackId, Pack>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pack: Pack) {
        self.packs.insert(pack.id.clone(), pack);
    }

    pub fn remove(&mut self, id: &PackId) -> Option<Pack> {
        self.packs.shift_remove(id)
    }

    pub fn get(&self, id: &PackId) -> Option<&Pack> {
        self.packs.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pack> {
        self.packs.values()
    }

    pub fn len(&self) -> usize {
        self.packs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packs.is_empty()
    }

    pub async fn load_from_config(config: config::Sticker) -> Self {
        if !config.enabled {
            return Self::new();
        }

        let fetches = config.packs.into_iter().map(|entry| async move {
            let url = entry.url;
            match super::fetch::fetch_pack(url.clone()).await {
                Ok(pack) => {
                    let pack = cache_pack_images(pack).await;
                    Some(pack)
                }
                Err(err) => {
                    log::warn!("failed to load sticker pack from {url}: {err}");
                    None
                }
            }
        });

        let results = futures::future::join_all(fetches).await;

        let mut registry = Self::new();
        for pack in results.into_iter().flatten() {
            registry.insert(pack);
        }
        registry
    }
}

/// Download all image files for a pack (cover + every sticker) into the
/// sticker cache directory, attaching the resolved local paths to the Pack.
/// Failed downloads are silently skipped — the sticker just won't show a
/// thumbnail in the picker.
async fn cache_pack_images(mut pack: Pack) -> Pack {
    let pack_dir = super::cache::pack_cache_dir(pack.id.as_str());

    // Cover
    if let Some(cover_file) = pack.manifest.cover.clone() {
        if let Ok(cover_url) = pack.base_url.join(&cover_file) {
            let dest = pack_dir.join(&cover_file);
            pack.cover_path =
                super::cache::ensure_cached(cover_url, dest).await;
        }
    }

    // Stickers in parallel
    let fetches = pack.manifest.stickers.iter().map(|s| {
        let id = s.id.clone();
        let file = s.file.clone();
        let base = pack.base_url.clone();
        let dest = pack_dir.join(&file);
        async move {
            let url = base.join(&file).ok()?;
            let path = super::cache::ensure_cached(url, dest).await?;
            Some((id, path))
        }
    });

    let results = futures::future::join_all(fetches).await;
    pack.sticker_paths = results
        .into_iter()
        .flatten()
        .collect::<BTreeMap<String, PathBuf>>();

    pack
}
