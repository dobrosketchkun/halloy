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

    pub fn get_mut(&mut self, id: &PackId) -> Option<&mut Pack> {
        self.packs.get_mut(id)
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

    /// Swap this pack one position earlier in the list. No-op if not found
    /// or already first.
    pub fn move_up(&mut self, id: &PackId) -> bool {
        match self.packs.get_index_of(id) {
            Some(idx) if idx > 0 => {
                self.packs.swap_indices(idx, idx - 1);
                true
            }
            _ => false,
        }
    }

    /// Swap this pack one position later in the list. No-op if not found or
    /// already last.
    pub fn move_down(&mut self, id: &PackId) -> bool {
        match self.packs.get_index_of(id) {
            Some(idx) if idx + 1 < self.packs.len() => {
                self.packs.swap_indices(idx, idx + 1);
                true
            }
            _ => false,
        }
    }

    /// Fetch a single pack by URL and add it to the live registry.
    /// Used by the manager modal when the user pastes a new pack URL.
    pub async fn add_pack_from_url(
        &mut self,
        url: url::Url,
    ) -> Result<PackId, super::Error> {
        let pack = super::fetch::fetch_pack(url).await?;
        let cached = cache_pack_images(pack).await;
        let id = cached.id.clone();
        self.insert(cached);
        Ok(id)
    }

    pub async fn load_from_config(config: config::Sticker) -> Self {
        if !config.enabled {
            return Self::new();
        }

        // Fetch all manifests in parallel (cheap — small JSON), then
        // sequentially disambiguate + cache images in config order so the
        // first-configured pack wins its unsuffixed id deterministically.
        let entries: Vec<(url::Url, Option<String>)> = config
            .packs
            .iter()
            .map(|e| (e.url.clone(), e.label.clone()))
            .collect();
        let fetches = config
            .packs
            .into_iter()
            .map(|entry| super::fetch::fetch_pack(entry.url));
        let results = futures::future::join_all(fetches).await;

        let mut registry = Self::new();
        for ((url, label), result) in entries.into_iter().zip(results.into_iter()) {
            match result {
                Ok(mut pack) => {
                    super::disambiguate_local_id(&mut pack, &url, &registry);
                    pack.label = label;
                    let cached = cache_pack_images(pack).await;
                    registry.insert(cached);
                }
                Err(err) => {
                    log::warn!(
                        "failed to load sticker pack from {url}: {err}"
                    );
                }
            }
        }
        registry
    }
}

/// Download all image files for a pack (cover + every sticker) into the
/// sticker cache directory, attaching the resolved local paths to the Pack.
/// Failed downloads are silently skipped — the sticker just won't show a
/// thumbnail in the picker.
pub async fn cache_pack_images(mut pack: Pack) -> Pack {
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
