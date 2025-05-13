# Simple RSS

A very simple terminal based RSS Reader.

> [!NOTE]
> This is still a very unfinished software with a lot of features missing.

## Usage

You can run the reader with

```sh
cargo run --release
```

or install and run it with

```sh
cargo install --path .
simple-rss
```

You can specify the list of RSS feeds at `~/.config/simple-rss`. This should be a file where each line is
a link to a feed. For example:

```text
https://feed-one.com/feed.xml
https://feed-two.com/atom.xml
```

### Shortcuts

- Move around with <kbd>Up</kbd> and <kbd>Down</kbd> arrows or vim motions <kbd>j</kbd> and <kbd>k</kbd>.
- Open item with <kbd>Enter</kbd>.
- Toggle if item is read with <kbd>Space</kbd>.
- Open item in browser with <kbd>o</kbd>.
- Move back or exit with <kbd>Escape</kbd> or <kbd>q</kbd>.

## TODO List

A list of things that need to be done:

- [ ] Cache rendered text in item list.
- [x] Improve keyboard navigation.
- [ ] Loading status, animations and progress reporting.
- [ ] Error reporting.
- [ ] Renaming channels
- [ ] Better channel management (adding, removing, updating channels).
- [ ] Help menu.
- [ ] Improve HTML rendering.

## License

The project is licensed under the [MIT License](LICENSE).
