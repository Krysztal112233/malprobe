# Malprobe

A VirusTotal-like virus scanning platform for simple malicious file scanning.

## Overview

Malprobe provides file upload and scanning capabilities, helping users quickly detect whether a file is malicious.

## Tech Stack

- `axum` and `mimalloc` as the core web infrastructure
- `sea-orm` for database operations
- ClamAV with prebaked signature databases for malware scanning
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

## ClamAV

`docker compose up -d --build` also starts a ClamAV service built from the
`clamav` target of the shared `Dockerfile`. The official databases
(`freshclam`) and the unofficial signature databases (`clamav-unofficial-sigs`,
including Sanesecurity and URLhaus) are baked into the image at build time.
The container runs only clamd and never updates its databases online.

Refresh the databases by rebuilding the image:

```bash
docker compose build clamav
docker compose up -d clamav
```
