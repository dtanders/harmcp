---
name: analyze-har
description: Use harmcp to investigate a HAR (HTTP Archive) file — finding slow requests, errors, payload contents, and initiator stacks without loading the whole file into memory.
---

# Analyzing HAR Files with harmcp

`harmcp` is a CLI tool for extracting structured data from HAR log files. It streams entries without loading the full file into memory, making it suitable for large captures.

## Basic usage

```
harmcp <file.har> <command> [options]
```

## Start with a list overview

Always begin by listing the entries to understand what's in the file:

```bash
harmcp capture.har list
```

This prints a fixed-width table with columns: IDX, METHOD, STATUS, URL, MIME, SIZE, TIME(ms).
The IDX column is the zero-based index you pass to detail commands.

For machine-readable output, use `--format json` (one JSON object per line) or `--format tsv`:

```bash
harmcp --format json capture.har list
harmcp --format tsv  capture.har list
```

## Filter the list

Narrow down to entries of interest before diving into details.

| Goal | Flag | Example |
|---|---|---|
| Specific method | `--method` | `--method POST` |
| Status class | `--status` | `--status 4xx`, `--status 5xx`, `--status 200-299` |
| URL substring | `--url` | `--url /api/` |
| URL regex | `--url-regex` | `--url-regex 'auth|login|token'` |
| MIME type | `--mime` | `--mime json`, `--mime image` |
| Response size | `--min-size` / `--max-size` | `--min-size 100000` |

Filters combine with AND. Find large failed JSON responses:

```bash
harmcp capture.har list --status 5xx --mime json --min-size 1000
```

## Inspect a specific entry

Once you have an IDX, use detail commands to dig in.

```bash
harmcp capture.har headers 4    # request + response headers
harmcp capture.har body     4   # request payload + response body
harmcp capture.har timings  4   # blocked/dns/connect/send/wait/receive breakdown
harmcp capture.har stack    4   # JS initiator call stack (if present)
harmcp capture.har all      4   # everything above in one output
```

Detail commands also support `--format json` for structured output:

```bash
harmcp --format json capture.har all 4
```

## Common investigation workflows

### Find the slowest requests

```bash
harmcp --format tsv capture.har list | sort -t$'\t' -k7 -rn | head -10
```

The last TSV column is `time_ms`; sort descending to surface slow entries.

### Find all errors and inspect the first one

```bash
# List errors
harmcp capture.har list --status 4xx --status 5xx

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
harmcp --format json capture.har list --url /api/ | \
  while read line; do
    idx=$(echo "$line" | jq -r '.index')
    harmcp --format json capture.har headers "$idx" | jq '.requestHeaders[] | select(.name | test("auth|cookie|token"; "i"))'
  done
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
  "headers": { "requestHeaders": [...], "responseHeaders": [...] },
  "body":    { "requestBody": null, "responseBody": "[{\"id\":1}]" },
  "timings": { "blocked": 1.0, "dns": 2.0, "connect": 3.0, "send": 0.5, "wait": 100.0, "receive": 16.9, "total": 123.4 },
  "stack":   { "type": "script", "callFrames": [{"functionName": "fetchUsers", "url": "app.js", "lineNumber": 42}] }
}
```
