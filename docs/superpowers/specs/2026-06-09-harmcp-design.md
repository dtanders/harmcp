# harmcp Design Spec
_2026-06-09_

## Overview

`harmcp` is a Rust command-line tool for extracting structured data from HAR (HTTP Archive) log files without loading the entire file into memory. It is designed for use by human operators and AI agents alike.

---

## Architecture

### Module Structure

```
harmcp/
├── src/
│   ├── main.rs           # CLI entry point, clap setup, dispatches to commands
│   ├── cli.rs            # Clap structs: Args, Commands, Filters, OutputFormat
│   ├── har/
│   │   ├── mod.rs        # Re-exports
│   │   ├── types.rs      # Serde structs: Entry, Request, Response, Timings, Initiator
│   │   └── stream.rs     # HarStream — BufReader-backed Iterator<Item = Result<Entry>>
│   ├── filter.rs         # Pure predicate: (&Entry, &Filters) -> bool
│   ├── output/
│   │   ├── mod.rs        # Dispatch to formatter by OutputFormat variant
│   │   ├── table.rs      # Fixed-width column-aligned table renderer
│   │   ├── tsv.rs        # Tab-separated output
│   │   └── json.rs       # JSON output via serde_json::to_writer
│   └── commands/
│       ├── list.rs       # Streams entries, applies filter, emits rows
│       └── detail.rs     # Shared logic for headers/body/timings/stack/all
```

`HarStream` is the core abstraction. All commands compose over it. `filter.rs` is pure (no I/O). Each command module takes `HarStream + Filters + OutputFormat` and is independently testable.

### Streaming Strategy

1. Open file as `BufReader<File>`
2. Use `serde_json::Deserializer::from_reader` with a custom `Visitor` to navigate to `log.entries`
3. `HarStream` implements `Iterator<Item = Result<Entry>>` — each `next()` deserializes exactly one entry and returns it; entries are dropped immediately after processing
4. For detail commands, `.nth(index)` skips entries cheaply without accumulating a Vec

---

## CLI Interface

```
harmcp <file.har> <COMMAND> [OPTIONS]
```

### Commands

| Command | Description |
|---|---|
| `list` | List all entries (with optional filtering and column selection) |
| `headers <index>` | Request + response headers for entry N |
| `body <index>` | Request payload + response body for entry N |
| `timings <index>` | Timing breakdown (blocked, dns, connect, send, wait, receive) |
| `stack <index>` | Initiator call stack for entry N |
| `all <index>` | All of the above for entry N |

### Global Options

| Flag | Description |
|---|---|
| `--format <fmt>` | `table` (default) \| `tsv` \| `json` |

### List Options

| Flag | Description |
|---|---|
| `--columns <col,...>` | Columns to display. Default: `index,method,status,url,mime,size,time` |
| `--method <method>` | Exact match, case-insensitive (e.g. `GET`, `post`) |
| `--status <pattern>` | Exact (`200`), wildcard (`4xx`, `20x`), or range (`400-499`) |
| `--url <pattern>` | Substring match (case-insensitive) |
| `--url-regex <pattern>` | Rust regex match against full URL |
| `--mime <pattern>` | Substring match (e.g. `json` matches `application/json`) |
| `--min-size <bytes>` | Minimum response body size |
| `--max-size <bytes>` | Maximum response body size |

### Default Column Set

`index | method | status | url | mime | size | time`

Column widths are fixed (not auto-sized to content) to support streaming output.

---

## HAR Data Model

Relevant HAR structure:

```json
{ "log": { "entries": [
  {
    "startedDateTime": "...",
    "time": 123.4,
    "request":  {
      "method": "GET",
      "url": "https://...",
      "headers": [{ "name": "...", "value": "..." }],
      "postData": { "text": "..." }
    },
    "response": {
      "status": 200,
      "statusText": "OK",
      "headers": [{ "name": "...", "value": "..." }],
      "content": { "size": 1234, "mimeType": "application/json", "text": "..." }
    },
    "timings": {
      "blocked": 0, "dns": 1.2, "connect": 3.4,
      "send": 0.1, "wait": 45.6, "receive": 2.3
    },
    "_initiator": {
      "type": "script",
      "stack": { "callFrames": [{ "functionName": "...", "url": "...", "lineNumber": 42 }] }
    }
  }
]}}
```

`_initiator` is a Chrome-specific extension. Modeled as `Option<Initiator>` so the tool is compatible with spec-compliant HAR files that omit it. All other optional fields follow the same pattern.

---

## Output Format

### `list`

Rows streamed to stdout as each entry is parsed — no full-result buffering.

- `table`: fixed-width columns, header row on first line
- `tsv`: tab-separated, header row on first line
- `json`: newline-delimited JSON objects (one per entry), enabling downstream streaming with `jq`

### Detail commands (`headers`, `body`, `timings`, `stack`)

A structured document per command.

- `table`/`tsv`: section headers + key-value rows
- `json`: single JSON object with relevant fields

### `all`

- `table`/`tsv`: each section separated by a blank line
- `json`: single object with keys `headers`, `body`, `timings`, `stack`

---

## Error Handling

| Condition | Behavior |
|---|---|
| File not found / unreadable | Message to stderr, exit 1 |
| Not valid JSON / not a HAR file | Message to stderr, exit 1 |
| Index out of range | "entry N not found (file has X entries)" to stderr, exit 1 |
| Missing optional field (`_initiator`, etc.) | Print empty/null, not an error |
| Malformed individual entry | Warn to stderr, skip entry, continue streaming |

---

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` (derive) | CLI argument parsing |
| `serde` + `serde_json` | HAR deserialization + JSON output |
| `comfy-table` | Table formatting |

---

## Out of Scope (v1)

- Byte-offset index / random-access caching across invocations
- Diffing two HAR files
- Writing/modifying HAR files
- MCP server mode
