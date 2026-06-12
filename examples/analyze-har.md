---
name: analyze-har
description: Use harmcp to investigate a HAR (HTTP Archive) file — finding slow requests, errors, payload contents, and initiator stacks without loading the whole file into memory.
---

# Analyzing HAR Files with harmcp

`harmcp` is a CLI tool for extracting structured data from HAR log files. It streams entries without loading the full file into memory, making it suitable for large captures.

## Basic usage

```
harmcp <file.har> <command> [options]
harmcp - <command> [options]   # read from stdin
```

## Start with a summary

Always begin with `summary` to understand what's in the file before narrowing down:

```bash
harmcp capture.har summary
```

This shows total entry count, status class breakdown, top domains and MIME types, and the slowest/largest entry. Takes a single streaming pass. Accepts all filter flags.

For large files or remote access, pipe via stdin:
```bash
gunzip -c big.har.gz | harmcp - summary
```

## List entries

After sizing up the file, list entries to find the ones you want:

```bash
harmcp capture.har list
```

This prints a fixed-width table with columns: IDX, METHOD, STATUS, URL, MIME, SIZE, TIME(ms).
The IDX column is the zero-based index you pass to detail commands.

For machine-readable output, use `--format json` (one JSON object per line) or `--format tsv`:

```bash
harmcp --format json capture.har list
harmcp --format tsv capture.har list
```

## Filter the list

Narrow down to entries of interest before diving into details.

| Goal | Flag | Example |
|---|---|---|
| Specific method | `--method` | `--method POST` |
| Status class | `--status` | `--status 4xx`, `--status 5xx`, `--status 200-299` |
| URL substring | `--url` | `--url /api/` |
| URL regex | `--url-regex` | `--url-regex 'auth\|login\|token'` |
| MIME type | `--mime` | `--mime json`, `--mime image` |
| Response size | `--min-size` / `--max-size` | `--min-size 100000` |
| No media | `--no-media` | excludes image, video, audio, font |
| No CSS | `--no-css` | |
| No media or CSS | `--no-assets` | shorthand for both |
| Exclude URL | `--not-url` | `--not-url telemetry` |
| Exclude MIME | `--not-mime` | `--not-mime html` |
| Exclude status | `--not-status` | `--not-status 3xx` |
| Exclude method | `--not-method` | `--not-method OPTIONS` |
| Time window | `--after` / `--before` | `--after 2024-06-01`, `--before 2024-06-02T12:00:00Z` |
| Duration | `--min-time` / `--max-time` | `--min-time 1000` (ms) |
| Request header | `--header` | `--header authorization`, `--header content-type=json` |
| Response header | `--resp-header` | `--resp-header cache-control=no-store` |
| Page scope | `--page` | `--page page_1` (see `pages` command) |

Filters combine with AND. Find large failed JSON responses:

```bash
harmcp capture.har list --status 5xx --mime json --min-size 1000
```

## Sort and limit

```bash
# Top 10 slowest requests
harmcp capture.har list --sort time --desc --limit 10

# Largest responses
harmcp capture.har list --sort size --desc --limit 10

# First 5 matching entries (stops streaming early, no buffering)
harmcp capture.har list --url /api/ --limit 5
```

## Inspect a specific entry

Once you have an IDX, use detail commands to dig in. Multiple indices are resolved in one pass:

```bash
harmcp capture.har headers  4        # request + response headers
harmcp capture.har body     4        # request payload + response body
harmcp capture.har timings  4        # blocked/dns/connect/send/wait/receive breakdown
harmcp capture.har stack    4        # JS initiator call stack (if present)
harmcp capture.har cookies  4        # request + response cookies
harmcp capture.har info     4        # timestamp, server IP, redirect, sizes, query params
harmcp capture.har ws       4        # WebSocket messages (if entry is a WebSocket)
harmcp capture.har all      4        # everything above in one output
harmcp capture.har headers  3 7 9    # multiple entries, single streaming pass
```

To save a binary response body to disk (base64 bodies are decoded automatically):
```bash
harmcp capture.har body 5 --output response.bin
harmcp capture.har body 2 --output logo.png
```

Detail commands also support `--format json` for structured output:

```bash
harmcp --format json capture.har all 4
```

## Pages

List pages recorded in the HAR (`log.pages`) and filter entries by page:

```bash
harmcp capture.har pages
harmcp capture.har list --page page_1
harmcp capture.har summary --page page_1
```

## Common investigation workflows

### Find the slowest requests

```bash
harmcp capture.har list --sort time --desc --limit 10
```

### Find all errors and inspect the first one

```bash
# List errors
harmcp capture.har list --status 400-599

# Inspect entry 7
harmcp capture.har all 7
```

### Trace what triggered a request

```bash
harmcp capture.har stack 12
```

If the HAR was captured from a browser with DevTools, this shows the JavaScript call stack that initiated the request — function name, source file, and line number.

### Check what a POST sent

```bash
harmcp capture.har body 3
```

Prints the raw request body (JSON, form data, etc.) and the response body.

### Scan for auth tokens in headers

```bash
# List all entries that have an Authorization request header
harmcp capture.har list --header authorization

# Entries with a specific cookie
harmcp capture.har list --header cookie=session_id

# Inspect headers for a specific entry
harmcp capture.har headers 4
```

### Scope investigation to a page

```bash
# See what pages are in the file
harmcp capture.har pages

# Investigate only entries from a specific page
harmcp capture.har list --page page_1
harmcp capture.har summary --page page_1 --no-assets
```

## Error handling

- **Exit 0** — success (even if no entries match a filter)
- **Exit 1** — file not found, invalid HAR structure, index out of range, bad regex
- Error messages go to stderr; structured output goes to stdout

If an entry is malformed, harmcp warns on stderr and skips it rather than aborting.

## Output reference

### list (table)
```
IDX    METHOD   STATUS  URL                                                           MIME                            SIZE        TIME(ms)
------------------------------------------------------------------------------------------------------------------------------------------
0      GET      200     https://example.com/api/users                                 application/json                512         123.4
```

### list (--format json)
One object per line — safe to pipe into `jq`:
```json
{"index":0,"method":"GET","status":200,"url":"https://example.com/api/users","mime":"application/json","size":512,"time_ms":123.4}
```

### all (--format json)
Single object with nested sections:
```json
{
  "index": 0,
  "method": "GET",
  "url": "https://example.com/api/users",
  "status": 200,
  "statusText": "OK",
  "headers": { "requestHeaders": [...], "responseHeaders": [...] },
  "body":    { "requestBody": null, "responseBody": "[{\"id\":1}]" },
  "timings": { "blocked": 1.0, "dns": 2.0, "connect": 3.0, "send": 0.5, "wait": 100.0, "receive": 16.9, "total": 123.4 },
  "stack":   { "type": "script", "callFrames": [{"functionName": "fetchUsers", "url": "app.js", "lineNumber": 42}] },
  "cookies": { "requestCookies": [], "responseCookies": [...] },
  "info":    { "startedDateTime": "...", "serverIPAddress": "...", "queryString": [...] }
}
```
