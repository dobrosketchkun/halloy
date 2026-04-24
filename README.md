# Halloy - IRC Client

<!-- === halloy-stickers fork: BEGIN === -->
> ### 📎 This is a fork — `stickerpacks`
>
> This branch adds **Telegram-style sticker packs** to halloy. Packs are just a `pack.json` manifest + folder of images on any public URL (typically a GitHub repo). Other halloy clients render them inline; non-halloy IRC clients see a clickable image link.
>
> **Quick tour:**
> - **Ctrl+Shift+S** → sticker picker (press-and-hold any sticker to preview zoomed)
> - **Ctrl+Shift+P** → pack manager (add/remove/reorder/rename, writes back to `config.toml`)
> - Click any sticker in chat → pack-info modal with a "Copy pack URL" button for sharing
> - Full config reference: [`docs/configuration/sticker.md`](./docs/configuration/sticker.md)
>
> Everything below is from upstream halloy and still applies — the fork adds functionality without removing any.
<!-- === halloy-stickers fork: END === -->

<img src="./assets/banner.png" alt="banner" title="Icon by Rune Seir">

![halloy](./assets/animation.gif)

Halloy is an open-source IRC client for Mac, Windows, and Linux, focused on being simple and fast.

Documentation: [halloy.chat](https://halloy.chat)

Join **#halloy** on libera.chat if you have questions or need help.

## Installation

[Installation documentation](https://halloy.chat/installation.html)

<a href="https://repology.org/project/halloy/versions">
    <img src="https://repology.org/badge/vertical-allrepos/halloy.svg" alt="Packaging status">
</a>

Halloy is also available from [Flathub](https://flathub.org/apps/org.squidowl.halloy) and [Snap Store](https://snapcraft.io/halloy).

## IRCv3 Capabilities

We strive to be a leading irc client with a rich ircv3 feature set. currently supported capabilities:

- [account-notify](https://ircv3.net/specs/extensions/account-notify)
- [away-notify](https://ircv3.net/specs/extensions/away-notify)
- [batch](https://ircv3.net/specs/extensions/batch)
- [cap-notify](https://ircv3.net/specs/extensions/capability-negotiation.html#cap-notify)
- [channel-context](https://ircv3.net/specs/client-tags/channel-context)
- [chathistory](https://ircv3.net/specs/extensions/chathistory)
- [chghost](https://ircv3.net/specs/extensions/chghost)
- [echo-message](https://ircv3.net/specs/extensions/echo-message)
- [extended-join](https://ircv3.net/specs/extensions/extended-join)
- [invite-notify](https://ircv3.net/specs/extensions/invite-notify)
- [labeled-response](https://ircv3.net/specs/extensions/labeled-response)
- [message-tags](https://ircv3.net/specs/extensions/message-tags)
- [Monitor](https://ircv3.net/specs/extensions/monitor)
- [msgid](https://ircv3.net/specs/extensions/message-ids)
- [multi-prefix](https://ircv3.net/specs/extensions/multi-prefix)
- [multiline](https://ircv3.net/specs/extensions/multiline)
- [react](https://ircv3.net/specs/client-tags/react.html)
- [read-marker](https://ircv3.net/specs/extensions/read-marker)
- [sasl-3.1](https://ircv3.net/specs/extensions/sasl-3.1)
- [server-time](https://ircv3.net/specs/extensions/server-time)
- [setname](https://ircv3.net/specs/extensions/setname.html)
- [Standard Replies](https://ircv3.net/specs/extensions/standard-replies)
- [typing](https://ircv3.net/specs/client-tags/typing)
- [userhost-in-names](https://ircv3.net/specs/extensions/userhost-in-names)
- [`UTF8ONLY`](https://ircv3.net/specs/extensions/utf8-only)
- [`WHOX`](https://ircv3.net/specs/extensions/whox)
- [`soju.im/bouncer-networks`](https://codeberg.org/emersion/soju/src/branch/master/doc/ext/bouncer-networks.md)
- [`soju.im/filehost`](https://codeberg.org/emersion/soju/src/branch/master/doc/ext/filehost.md)
<!-- === halloy-stickers fork: BEGIN === -->
- `+halloy.chat/sticker` — vendor-prefixed client tag for sticker messages (fork-specific, not an upstream IRCv3 spec)
<!-- === halloy-stickers fork: END === -->

## Why?

<a href="https://xkcd.com/1782/">
  <img src="https://imgs.xkcd.com/comics/team_chat.png" title="2078: He announces that he's finally making the jump from screen+irssi to tmux+weechat.">
</a>

## Contributing

See the [contributing guide](https://halloy.chat/contributing) to get started.

## License

Halloy is released under the GPL-3.0 License. For more details, see the [LICENSE](LICENSE) file.

## Contact

For any questions, suggestions, or issues, please open an issue on the [GitHub repository](https://github.com/squidowl/halloy/issues).

<a href="https://github.com/iced-rs/iced">
  <img src="https://gist.githubusercontent.com/hecrj/ad7ecd38f6e47ff3688a38c79fd108f0/raw/74384875ecbad02ae2a926425e9bcafd0695bade/color.svg" width="130px">
</a>
