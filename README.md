# Malprobe

A VirusTotal-like virus scanning platform for simple malicious file scanning.

## Overview

Malprobe provides file upload and scanning capabilities, helping users quickly detect whether a file is malicious.

## Tech Stack

- `axum` and `mimalloc` as the core web infrastructure
- `sea-orm` for database operations
- One-click deployment via compose

## Workspace Structure

- `crates/malprobe` — main backend service (API entrypoint)
- `crates/malprobe-common` — shared error handling
- `crates/malprobe-vo` — pure response value objects (VOs)
- `crates/migration` — database migrations

## Running

```bash
docker compose up -d --build
```

The service listens on port `8000` by default and is configured via `malprobe.toml`.
