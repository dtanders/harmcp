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
```

### list

Print all entries as a table. Use filters to narrow down.

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

Filters combine with AND. Show only large server errors:
```
harmcp capture.har list --status 5xx --min-size 1000
```

**Column selection** — default columns are `index,method,status,url,mime,size,time`. Override with `--columns`:
```
harmcp capture.har list --columns index,url,time
```

### Detail commands

All take a zero-based entry index from the `IDX` column.

```
harmcp capture.har headers 0    # request + response headers
harmcp capture.har body     0   # request payload + response body
harmcp capture.har timings  0   # timing breakdown
harmcp capture.har stack    0   # JS initiator call stack
harmcp capture.har all      0   # everything above combined
```

**headers:**
```
=== Request Headers ===
Accept: application/json

=== Response Headers ===
Content-Type: application/json
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

[`examples/analyze-har.md`](examples/analyze-har.md) is a Claude Code skill. Copy it to your skills folder to install:

```bash
# global (all projects)
cp examples/analyze-har.md ~/.claude/skills/

# project-local
cp examples/analyze-har.md .claude/skills/
```

Once installed, invoke it with `/analyze-har` or `Skill({ skill: "analyze-har" })`.

## How it works

HAR files can be hundreds of megabytes. `harmcp` uses a custom `serde` visitor chain on `serde_json::Deserializer::from_reader` to walk `log.entries` as a token stream — only one entry is in memory at a time. Detail commands stop streaming after finding their target entry, draining the remainder cheaply via `IgnoredAny`. Malformed entries are warned and skipped rather than aborting the stream.

## Building

```
cargo build --release
```

Requires Rust stable (2021 edition).
