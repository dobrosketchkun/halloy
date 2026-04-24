# Sticker

Telegram-style sticker packs served from any public repo (typically GitHub).

Added by the **stickerpacks fork**. Packs are just a `pack.json` manifest and a folder of images at a public URL — no server-side infrastructure needed.

::: tip
The easiest way to manage packs is the in-app **Pack Manager** — press **Ctrl+Shift+P** to add, remove, rename, or reorder packs. Every change persists to `config.toml` automatically. The options below document what the manager writes.
:::

::: warning
The upstream halloy docs site (`halloy.chat`) does **not** cover this fork's options. This markdown file is the only reference. If you publish this fork on GitHub, the file renders there; otherwise read it locally in the repo.
:::

## Opening the picker

Press **Ctrl+Shift+S** anywhere to open the sticker picker. Click a sticker to send it to the current buffer. Press-and-hold any sticker (picker, chat, pack info) to preview it zoomed in — release anywhere other than the pressed sticker to cancel.

## `enabled`

Whether to load any sticker packs at all. Set to `false` to disable the feature entirely (packs stay configured but aren't fetched).

```toml
# Type: boolean
# Default: true

[sticker]
enabled = true
```

## `max_size_px`

Maximum display size of a rendered sticker in chat, in pixels (applied to both width and height). Images are scaled down to fit.

```toml
# Type: integer
# Default: 200

[sticker]
max_size_px = 200
```

## `max_manifest_bytes`

Reject `pack.json` responses larger than this many bytes. Defends against a malicious repo serving a huge manifest.

```toml
# Type: integer (bytes)
# Default: 65536 (64 KB)

[sticker]
max_manifest_bytes = 65536
```

## `max_image_bytes`

Reject individual sticker images larger than this many bytes during fetch. Failed stickers are silently skipped (no thumbnail in the picker).

```toml
# Type: integer (bytes)
# Default: 524288 (512 KB)

[sticker]
max_image_bytes = 524288
```

## `max_stickers_per_pack`

Keep at most this many stickers per pack. Extra stickers in the manifest are silently truncated.

```toml
# Type: integer
# Default: 120

[sticker]
max_stickers_per_pack = 120
```

## `[[sticker.packs]]`

Each subscribed pack is one `[[sticker.packs]]` entry. The URL can be any of:

- **Folder view** on GitHub: `https://github.com/user/repo/tree/main/mypack`
- **File view** on GitHub: `https://github.com/user/repo/blob/main/mypack/pack.json` (filename is stripped)
- **Raw CDN URL**: `https://raw.githubusercontent.com/user/repo/main/mypack/`

Halloy normalises all of these to the raw form internally for fetching.

```toml
[[sticker.packs]]
url = "https://github.com/user/stickers/tree/main/catgirl"
label = "Catgirl reactions"  # optional; defaults to pack.json's `name`

[[sticker.packs]]
url = "https://github.com/user/stickers/tree/main/yes"
```

### `label` (optional)

A local display name override. When set, it's shown everywhere in place of the `name` field from `pack.json`. Useful for disambiguating two packs that happen to share the same id/name in their manifests, and for personal shorthand.

## Pack repository structure

A pack is a folder containing:

- `pack.json` — the manifest
- One image file per sticker (PNG, WebP, JPG)
- Optionally a `cover.*` image used as the pack icon in the picker

```
my-sticker-repo/
└── mypack/
    ├── pack.json
    ├── cover.webp
    ├── 01.webp
    ├── 02.webp
    └── yes.webp
```

### `pack.json`

```json
{
  "id": "mypack",
  "name": "My Pack",
  "author": "Your Name",
  "description": "A short description.",
  "version": 1,
  "cover": "cover.webp",
  "stickers": [
    { "id": "01", "file": "01.webp", "emoji": "😐", "tags": ["neutral"] },
    { "id": "02", "file": "02.webp", "emoji": "😴💤", "tags": ["sleep", "tired"] },
    { "id": "yes", "file": "yes.webp", "emoji": "👍", "tags": ["yes", "ok"] }
  ]
}
```

- `id` — URL-safe identifier. Used in the wire-format tag when sending a sticker.
- `name` — human-readable display name.
- `emoji` — matches Telegram's data model: a string of one or more emoji associated with the sticker. All are used for search.
- `file` — either a **relative filename** resolved against the pack folder's URL (ordinary case, images live alongside `pack.json`), **or an absolute `http`/`https` URL** pointing anywhere else. This lets a pack author keep `pack.json` on GitHub while hosting individual stickers on a CDN, imgur, a shared image repo, etc. The `cover` field accepts the same two forms.

```json
{
  "id": "mix",
  "name": "Mixed hosting",
  "version": 1,
  "stickers": [
    { "id": "local",  "file": "01.webp",                          "emoji": "😐" },
    { "id": "cdn",    "file": "https://i.imgur.com/abc123.webp",  "emoji": "👍" },
    { "id": "shared", "file": "https://example.com/common/x.png", "emoji": "🎉" }
  ]
}
```

## Wire format

A sticker message goes out as a plain `PRIVMSG` whose body is the image URL, with an IRCv3 client tag attached:

```
@+halloy.chat/sticker=mypack/01 PRIVMSG #channel :https://raw.githubusercontent.com/user/stickers/main/mypack/01.webp
```

Non-halloy clients receive a clickable image link. Halloy clients see the rendered sticker inline and suppress the URL text.

## Sharing a pack

Click any sticker someone else sent in chat to open the **Pack Info** modal, which shows the pack's stickers and a **Copy pack URL** button. Paste that URL into your own manager (Ctrl+Shift+P) to subscribe.

## Files

**Sticker image cache** (re-downloadable, safe to delete):

* Windows: `%LOCALAPPDATA%\halloy\stickers\`
* Mac: `~/Library/Caches/halloy/stickers/`
* Linux: `$XDG_CACHE_HOME/halloy/stickers/` or `~/.cache/halloy/stickers/`

**Recents list** (persisted across restarts):

* Windows: `%AppData%\Roaming\halloy\sticker_recents.json`
* Mac: `~/Library/Application Support/halloy/sticker_recents.json` or `$HOME/.local/share/halloy/sticker_recents.json`
* Linux: `$XDG_DATA_HOME/halloy/sticker_recents.json` or `$HOME/.local/share/halloy/sticker_recents.json`
