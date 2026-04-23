use std::path::PathBuf;

use tokio::fs;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};
use url::Url;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config is not valid TOML: {0}")]
    Parse(#[from] toml_edit::TomlError),
}

/// Rewrite the `[[sticker.packs]]` entries in the user's config.toml to
/// match the given entries, preserving all comments, formatting, and other
/// sections. Atomic: writes to a temp file then renames.
///
/// Other `[sticker]` fields (enabled, max_size_px, …) are left untouched.
pub async fn save_packs(
    entries: Vec<(Url, Option<String>)>,
) -> Result<(), Error> {
    let path = Config::path();

    let existing = match fs::read_to_string(&path).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(Error::Io(e)),
    };

    let mut doc = if existing.is_empty() {
        DocumentMut::new()
    } else {
        existing.parse::<DocumentMut>()?
    };

    // Ensure the [sticker] table exists without clobbering sibling fields.
    if doc.get("sticker").is_none() {
        doc["sticker"] = Item::Table(Table::new());
    }

    let mut aot = ArrayOfTables::new();
    for (url, label) in &entries {
        let mut t = Table::new();
        t["url"] = value(url.to_string());
        if let Some(label) = label {
            t["label"] = value(label.clone());
        }
        aot.push(t);
    }
    doc["sticker"]["packs"] = Item::ArrayOfTables(aot);

    let tmp: PathBuf = path.with_extension("toml.tmp");
    fs::write(&tmp, doc.to_string()).await?;
    fs::rename(&tmp, &path).await?;
    Ok(())
}

/// Snapshot the current shared registry's packs (URL + optional user label)
/// and persist to config.toml. Every manager-modal mutation (add, remove,
/// reorder, rename) calls this after updating in-memory state.
pub async fn persist_registry() -> Result<(), Error> {
    let entries: Vec<(Url, Option<String>)> = super::with_shared(|reg| {
        reg.iter()
            .map(|p| {
                (
                    super::fetch::to_browseable_url(&p.base_url),
                    p.label.clone(),
                )
            })
            .collect()
    });
    save_packs(entries).await
}
