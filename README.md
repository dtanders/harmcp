# harmcp

Extract structured data from HAR (HTTP Archive) log files without loading the whole file into memory. Designed for use by both humans and AI agents.

## Install

**From a release** — download the binary for your platform from the [releases page](https://github.com/dtanders/harmcp/releases) and put it on your PATH.

**Canary** (latest passing build from `master`) — grab from the [canary release](https://github.com/dtanders/harmcp/releases/tag/canary).

**From source:**
```
cargo install --git https://github.com/dtanders/harmcp
```

## Usage

```
harmcp <file.har> <command> [options]
harmcp - <command> [options]   # read from stdin
```

Pass `-` as the file to read from stdin:
```bash
gunzip -c big.har.gz | harmcp - list
cat capture.har | harmcp - summary
```

## Commands

### summary

Get aggregate statistics in one pass. A good first step to size up a file.

```
$ harmcp capture.har summary

entries:      4
total bytes:  516
total time:   188.4 ms
time span:    2024-01-01T00:00:00.000Z .. 2024-01-01T00:00:03.000Z

status:  1xx 1   2xx 2   3xx 0   4xx 1   5xx 0   other 0

top mime types:
  (none)                                   1
  application/json                         1
  image/png                                1
  text/html                                1

top domains:
  example.com                              4

slowest:  [0] 123.4 ms  https://example.com/api/users?limit=10
largest:  [0] 512 bytes  https://example.com/api/users?limit=10
```

Respects all filter flags. JSON format produces `{"entries": N, "totalBytes": ..., "statusClasses": {...}, ...}`.

### list

Print matching entries as a table. Use filters to narrow down.

```
$ harmcp capture.har list

IDX    METHOD   STATUS  URL                                                           MIME                            SIZE        TIME(ms)
------------------------------------------------------------------------------------------------------------------------------------------
0      GET      200     https://example.com/api/users                                 application/json                512         123.4
1      POST     401     https://example.com/api/login                                 text/html                       0           50.0
```

**Filter flags:**

| Flag | Match | Examples |
|---|---|---|
| `--method` | exact, case-insensitive | `--method POST` |
| `--status` | exact, wildcard, or range | `--status 4xx` · `--status 200-299` · `--status 201` |
| `--url` | substring, case-insensitive | `--url /api/` |
| `--url-regex` | Rust regex | `--url-regex 'auth\|login'` |
| `--mime` | substring, case-insensitive | `--mime json` · `--mime image` |
| `--min-size` / `--max-size` | response body bytes | `--min-size 100000` |
| `--no-media` | exclude image, video, audio, font | |
| `--no-css` | exclude CSS | |
| `--no-assets` | exclude media + CSS (both of the above) | |
| `--not-url` | exclude URL substring, case-insensitive | `--not-url telemetry` |
| `--not-mime` | exclude MIME substring, case-insensitive | `--not-mime html` |
| `--not-status` | exclude status pattern | `--not-status 3xx` |
| `--not-method` | exclude method, case-insensitive | `--not-method OPTIONS` |
| `--after` | started at or after (RFC 3339 or YYYY-MM-DD) | `--after 2024-06-01` |
| `--before` | started before (RFC 3339 or YYYY-MM-DD) | `--before 2024-06-02T12:00:00Z` |
| `--min-time` / `--max-time` | total entry time in ms | `--min-time 1000` |
| `--header` | request header present or name=value substring (repeatable) | `--header authorization` · `--header content-type=json` |
| `--resp-header` | response header present or name=value substring (repeatable) | `--resp-header cache-control=no-store` |
| `--page` | page id from `pages` command | `--page page_1` |

Filters combine with AND. Show only large server errors:
```
harmcp capture.har list --status 5xx --min-size 1000
```

**Sorting and limiting:**

```bash
# Top 10 slowest requests
harmcp capture.har list --sort time --desc --limit 10

# Sort by size descending
harmcp capture.har list --sort size --desc

# First 5 matching entries (stops streaming early)
harmcp capture.har list --url /api/ --limit 5
```

`--sort` buffers all matching entries to sort them. `--limit` without `--sort` stops streaming early.

**Column selection** — default columns: `index,method,status,url,mime,size,time`. Add `start` for the timestamp:
```
harmcp capture.har list --columns index,url,time,start
```

### Detail commands

All detail commands accept one or more zero-based indices from the `IDX` column. Multiple indices are resolved in a single streaming pass.

```bash
harmcp capture.har headers  0 5 12   # request + response headers
harmcp capture.har body     0        # request payload + response body
harmcp capture.har timings  0        # timing breakdown
harmcp capture.har stack    0        # JS initiator call stack
harmcp capture.har cookies  0        # request + response cookies
harmcp capture.har info     0        # timestamp, status text, server IP, redirect, sizes, query params
harmcp capture.har ws       3        # WebSocket messages (_webSocketMessages)
harmcp capture.har all      0        # everything above combined
```

`body --output <file>` writes the decoded response body to a file instead of stdout (base64 bodies are decoded automatically; works for binary responses):
```bash
harmcp capture.har body 2 --output logo.png
```

**info:**
```
started:       2024-01-01T00:00:00.000Z
status:        200 OK
server ip:     93.184.216.34
page:          page_1

=== Query Parameters ===
limit = 10
```

**timings:**
```
blocked             1.0 ms
dns                 2.0 ms
connect             3.0 ms
send                0.5 ms
wait              100.0 ms
receive            16.9 ms
--------------------------
total             123.4 ms
```

### pages

List the pages recorded in the HAR (from `log.pages`):

```
$ harmcp capture.har pages

page_1  2024-01-01T00:00:00.000Z  Example Dashboard
```

Use `--page` in `list` and `summary` to scope to entries from one page:
```bash
harmcp capture.har list --page page_1
harmcp capture.har summary --page page_1
```

### Output formats

`--format` is a global flag, place it before the subcommand:

```
harmcp --format json capture.har list
harmcp --format tsv  capture.har list
```

- `table` (default) — fixed-width, human-readable
- `tsv` — tab-separated, header row included, pipe-friendly
- `json` — one JSON object per line for `list`; single JSON object for detail commands

JSON list output (one line per entry, safe to pipe into `jq`):
```json
{"index":0,"method":"GET","status":200,"url":"https://example.com/api/users","mime":"application/json","size":512,"time_ms":123.4}
```

## AI agent usage

[`skills/analyze-har/CLAUDE.md`](skills/analyze-har/CLAUDE.md) is a Claude Code skill. Install it with the built-in subcommand:

```bash
# project-local (.claude/skills/ in the current directory)
harmcp skill install

# global (~/.claude/skills/, available in all projects)
harmcp skill install --global
```

Once installed, invoke it with `/analyze-har` or `Skill({ skill: "analyze-har" })`.

## How it works

HAR files can be hundreds of megabytes. `harmcp` uses a custom `serde` visitor chain on `serde_json::Deserializer::from_reader` to walk `log.entries` as a token stream — only one entry is in memory at a time. Detail commands stop streaming after finding their target entry, draining the remainder cheaply via `IgnoredAny`. Malformed entries are warned and skipped rather than aborting the stream. `--sort` is the only mode that buffers (bodies are shed before buffering to limit memory).

## Building

```
cargo build --release
```

Requires Rust stable (2021 edition).
