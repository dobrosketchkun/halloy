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

    pub async fn load_from_config(config: &config::Sticker) -> Self {
        if !config.enabled {
            return Self::new();
        }

        let fetches = config.packs.iter().map(|entry| {
            let url = entry.url.clone();
            async move {
                match super::fetch::fetch_pack(url.clone()).await {
                    Ok(pack) => Some(pack),
                    Err(err) => {
                        log::warn!(
                            "failed to load sticker pack from {url}: {err}"
                        );
                        None
                    }
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
