use futures::StreamExt;
use url::Url;

use super::{Error, Pack, PackId, PackManifest};

pub async fn fetch_pack(base_url: Url) -> Result<Pack, Error> {
    let base_url = normalize_base_url(base_url);
    let manifest_url = base_url.join("pack.json")?;

    let config = super::shared_config();

    let response = reqwest::get(manifest_url).await?.error_for_status()?;
    let bytes = fetch_limited_bytes(response, config.max_manifest_bytes)
        .await
        .ok_or(Error::ManifestTooLarge {
            max: config.max_manifest_bytes,
        })?;

    let mut pack = parse_manifest(base_url, &bytes)?;

    // Truncate sticker list to the configured per-pack limit — defends
    // against a bloated pack filling the picker forever.
    if pack.manifest.stickers.len() > config.max_stickers_per_pack {
        pack.manifest
            .stickers
            .truncate(config.max_stickers_per_pack);
    }

    Ok(pack)
}

/// Read up to `max_bytes` from a response's body. Returns None if the body
/// exceeds the limit (either per Content-Length or during streaming).
/// Used to bound pack.json and sticker image downloads so a hostile repo
/// can't exhaust memory.
pub(super) async fn fetch_limited_bytes(
    response: reqwest::Response,
    max_bytes: usize,
) -> Option<bytes::Bytes> {
    if let Some(len) = response.content_length() {
        if len as usize > max_bytes {
            return None;
        }
    }

    let mut buf = bytes::BytesMut::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        if buf.len() + chunk.len() > max_bytes {
            return None;
        }
        buf.extend_from_slice(&chunk);
    }
    Some(buf.freeze())
}

/// Accept either a raw.githubusercontent.com URL or a github.com tree/blob URL
/// and produce the raw-form base URL that `base.join("pack.json")` can resolve
/// against. Non-GitHub URLs are passed through unchanged (with trailing slash).
fn normalize_base_url(url: Url) -> Url {
    if url.host_str() == Some("github.com") {
        let segments: Vec<&str> = url
            .path_segments()
            .map(|s| s.filter(|seg| !seg.is_empty()).collect())
            .unwrap_or_default();

        // github.com/USER/REPO/tree/BRANCH/PATH...  -> folder
        // github.com/USER/REPO/blob/BRANCH/PATH.../FILE -> file (drop last)
        if segments.len() >= 4
            && (segments[2] == "tree" || segments[2] == "blob")
        {
            let user = segments[0];
            let repo = segments[1];
            let kind = segments[2];
            let branch = segments[3];

            let path_segments: &[&str] = if kind == "blob" && segments.len() > 4
            {
                &segments[4..segments.len() - 1]
            } else {
                &segments[4..]
            };

            let mut raw_path = format!("/{user}/{repo}/{branch}");
            for seg in path_segments {
                raw_path.push('/');
                raw_path.push_str(seg);
            }
            raw_path.push('/');

            let mut rewritten = url.clone();
            let _ = rewritten.set_host(Some("raw.githubusercontent.com"));
            rewritten.set_path(&raw_path);
            return rewritten;
        }
    }

    ensure_trailing_slash(url)
}

pub fn parse_manifest(base_url: Url, bytes: &[u8]) -> Result<Pack, Error> {
    let manifest: PackManifest = serde_json::from_slice(bytes)?;
    let pack_id = PackId::new(manifest.id.as_str())
        .ok_or_else(|| Error::InvalidPackId(manifest.id.clone()))?;
    Ok(Pack::new(pack_id, base_url, manifest))
}

fn ensure_trailing_slash(mut url: Url) -> Url {
    if !url.path().ends_with('/') {
        let mut path = url.path().to_owned();
        path.push('/');
        url.set_path(&path);
    }
    url
}

/// Reverse of `normalize_base_url`: turn a raw.githubusercontent.com URL
/// back into the human-browseable github.com/tree form. Used for UI strings
/// like the "Copy pack URL" button — users shouldn't ever see or paste the
/// CDN URL. Non-github URLs pass through unchanged.
pub fn to_browseable_url(url: &Url) -> Url {
    if url.host_str() == Some("raw.githubusercontent.com") {
        let segments: Vec<&str> = url
            .path_segments()
            .map(|s| s.filter(|seg| !seg.is_empty()).collect())
            .unwrap_or_default();

        // path is /USER/REPO/BRANCH/PATH...
        if segments.len() >= 3 {
            let user = segments[0];
            let repo = segments[1];
            let branch = segments[2];

            let mut out = format!("https://github.com/{user}/{repo}/tree/{branch}");
            for seg in &segments[3..] {
                out.push('/');
                out.push_str(seg);
            }

            if let Ok(parsed) = Url::parse(&out) {
                return parsed;
            }
        }
    }
    url.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sticker::StickerId;

    const SAMPLE_JSON: &str = r#"{
        "id": "dsk",
        "name": "DSK",
        "version": 1,
        "stickers": [
            {"id": "01", "file": "01.webp", "emoji": "😐"}
        ]
    }"#;

    fn base() -> Url {
        Url::parse("https://example.com/packs/dsk/").unwrap()
    }

    #[test]
    fn parse_minimal_manifest() {
        let pack = parse_manifest(base(), SAMPLE_JSON.as_bytes()).unwrap();
        assert_eq!(pack.id.as_str(), "dsk");
        assert_eq!(pack.manifest.name, "DSK");
        assert_eq!(pack.manifest.stickers.len(), 1);
        assert_eq!(pack.manifest.stickers[0].emoji, "\u{1F610}");
    }

    #[test]
    fn emoji_string_preserved() {
        let json = r#"{
            "id": "dsk",
            "name": "DSK",
            "version": 1,
            "stickers": [{"id": "01", "file": "01.webp", "emoji": "😐🫥😶"}]
        }"#;
        let pack = parse_manifest(base(), json.as_bytes()).unwrap();
        assert_eq!(
            pack.manifest.stickers[0].emoji,
            "\u{1F610}\u{1FAE5}\u{1F636}"
        );
    }

    #[test]
    fn rejects_invalid_id() {
        let json =
            r#"{"id": "bad id", "name": "x", "version": 1, "stickers": []}"#;
        assert!(matches!(
            parse_manifest(base(), json.as_bytes()),
            Err(Error::InvalidPackId(_))
        ));
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(matches!(
            parse_manifest(base(), b"not json"),
            Err(Error::Json(_))
        ));
    }

    #[test]
    fn trailing_slash_added() {
        let url = Url::parse("https://example.com/packs/dsk").unwrap();
        assert_eq!(ensure_trailing_slash(url).path(), "/packs/dsk/");
    }

    #[test]
    fn trailing_slash_preserved() {
        let url = Url::parse("https://example.com/packs/dsk/").unwrap();
        assert_eq!(ensure_trailing_slash(url).path(), "/packs/dsk/");
    }

    #[test]
    fn github_tree_url_rewrites_to_raw() {
        let url =
            Url::parse("https://github.com/you/stickers/tree/main/dsk")
                .unwrap();
        assert_eq!(
            normalize_base_url(url).as_str(),
            "https://raw.githubusercontent.com/you/stickers/main/dsk/"
        );
    }

    #[test]
    fn github_tree_url_trailing_slash() {
        let url =
            Url::parse("https://github.com/you/stickers/tree/main/dsk/")
                .unwrap();
        assert_eq!(
            normalize_base_url(url).as_str(),
            "https://raw.githubusercontent.com/you/stickers/main/dsk/"
        );
    }

    #[test]
    fn github_blob_url_strips_filename() {
        let url = Url::parse(
            "https://github.com/you/stickers/blob/main/dsk/pack.json",
        )
        .unwrap();
        assert_eq!(
            normalize_base_url(url).as_str(),
            "https://raw.githubusercontent.com/you/stickers/main/dsk/"
        );
    }

    #[test]
    fn github_tree_root_folder() {
        let url =
            Url::parse("https://github.com/you/stickers/tree/main").unwrap();
        assert_eq!(
            normalize_base_url(url).as_str(),
            "https://raw.githubusercontent.com/you/stickers/main/"
        );
    }

    #[test]
    fn raw_url_passed_through() {
        let url = Url::parse(
            "https://raw.githubusercontent.com/you/stickers/main/dsk/",
        )
        .unwrap();
        assert_eq!(
            normalize_base_url(url).as_str(),
            "https://raw.githubusercontent.com/you/stickers/main/dsk/"
        );
    }

    #[test]
    fn non_github_url_only_gets_trailing_slash() {
        let url = Url::parse("https://example.com/stickers/dsk").unwrap();
        assert_eq!(
            normalize_base_url(url).as_str(),
            "https://example.com/stickers/dsk/"
        );
    }

    #[test]
    fn sticker_url_resolves_relative_to_base() {
        let pack = parse_manifest(base(), SAMPLE_JSON.as_bytes()).unwrap();
        let id = StickerId::new("01").unwrap();
        let url = pack.sticker_url(&id).unwrap();
        assert_eq!(url.as_str(), "https://example.com/packs/dsk/01.webp");
    }
}
