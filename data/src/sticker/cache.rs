use std::path::PathBuf;

use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use reqwest::header::IF_MODIFIED_SINCE;
use tokio::fs;
use url::Url;

use crate::environment;

pub fn stickers_cache_dir() -> PathBuf {
    environment::cache_dir().join("stickers")
}

pub fn pack_cache_dir(pack_id: &str) -> PathBuf {
    stickers_cache_dir().join(pack_id)
}

/// Fetch `url` to `dest`, using `If-Modified-Since` when a cached copy
/// already exists. GitHub's raw CDN honours the header and replies `304 Not
/// Modified` for unchanged files — cheap round-trip, no body transfer.
/// Any other successful response overwrites the cached file so the user
/// sees updated stickers after editing their pack repo.
pub async fn ensure_cached(url: Url, dest: PathBuf) -> Option<PathBuf> {
    let mut request = reqwest::Client::new().get(url.clone());

    if let Ok(metadata) = fs::metadata(&dest).await
        && let Ok(mtime) = metadata.modified()
    {
        let http_date = DateTime::<Utc>::from(mtime)
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();
        request = request.header(IF_MODIFIED_SINCE, http_date);
    }

    let response = request.send().await.ok()?;

    if response.status() == StatusCode::NOT_MODIFIED && dest.exists() {
        return Some(dest);
    }

    let bytes = response.error_for_status().ok()?.bytes().await.ok()?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await.ok()?;
    }
    fs::write(&dest, &bytes).await.ok()?;
    Some(dest)
}
