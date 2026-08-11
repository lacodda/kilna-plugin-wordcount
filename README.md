# kilna-plugin-wordcount

**Counts what a draft actually contains** — words, lines and an estimated reading time — and writes them into the work's fields in [kilna](https://github.com/lacodda/kilna).

The first plugin written against kilna's plugin protocol, and the thing that proved the protocol works.

## Install

Download the binary for your platform from [Releases](https://github.com/lacodda/kilna-plugin-wordcount/releases) and put it in kilna's `plugins` directory, beside the workspace database. kilna finds anything named `kilna-plugin-*` there or on your `PATH`.

kilna shows a **Count words** button on every work once it is installed.

## What it writes

| Field | Meaning |
|---|---|
| `words` | Whitespace-separated runs containing at least one letter or digit, so a lone dash is not counted |
| `lines` | Non-empty lines |
| `reading_minutes` | At 200 words per minute, rounded up, never zero |

It measures the longest body the work has. A song's lyrics rather than its style prompt; a chapter's text rather than its outline. Picking by role name would tie the plugin to one craft.

## The protocol

A plugin is an ordinary executable. kilna runs it twice:

```sh
kilna-plugin-wordcount --manifest   # what it offers, as JSON on stdout
kilna-plugin-wordcount run          # invocation as JSON on stdin, outcome on stdout
```

The manifest declares a protocol version, a name, and the commands it contributes:

```jsonc
{
  "protocol_version": 1,
  "name": "wordcount",
  "version": "0.1.0",
  "commands": [
    { "key": "count", "label": "Count words", "target": "work" }
  ]
}
```

An invocation arrives as `{"command": "count", "target": "work", "subject": {…}}`, where `subject` is the row plus a `bodies` object holding each version role's latest text. The reply may carry `meta` (merged into the row — a plugin can add and overwrite its own keys but never clear the rest), `message` (shown to the user), or `error`.

Plugins live for the duration of one call. Nothing here starts a service.

## Build

```sh
cargo build --release
cargo test
```

Requires Rust 1.85 or newer.

## License

MIT
