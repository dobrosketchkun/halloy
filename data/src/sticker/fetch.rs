use url::Url;

use super::{Error, Pack, PackId, PackManifest};

pub async fn fetch_pack(base_url: Url) -> Result<Pack, Error> {
    let base_url = normalize_base_url(base_url);
    let manifest_url = base_url.join("pack.json")?;

    let bytes = reqwest::get(manifest_url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    parse_manifest(base_url, &bytes)
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
