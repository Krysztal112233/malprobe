# Malprobe

VirusTotal-like file scanning platform: axum backend + ClamAV scanning, tasks
distributed via pgmq (Postgres message queue) to a separate worker process.

## Architecture

```
POST /files → store file → INSERT files(pending) → pgmq.send("scan")
    → worker read_with_poll → ClamAV INSTREAM → UPDATE files → pgmq.delete
```

- `crates/malprobe` — backend API (axum); uploads enqueue scan tasks
- `crates/malprobe-worker` — consumes tasks, runs ClamAV, updates results
- `crates/malprobe-config` — all config structs + loading (`malprobe.toml` + `MALPROBE_*` env)
- `crates/malprobe-common` / `crates/malprobe-vo` / `crates/migration` — errors / VOs / SeaORM migrations

## Rules

- **Migrations**: generate with the SeaORM CLI (`cargo run -p migration -- generate <name>`),
  never hand-write. Raw SQL goes in `<migration>/up.sql` + `down.sql`, embedded via `include_str!`.
  Directory-style migrations must `impl MigrationName` manually — `DeriveMigrationName`
  derives from the file stem, which is `"mod"` inside a directory.
- **Config**: define config types only in `malprobe-config`; never redefine in service crates.
- **pgmq**: keep the crate at 0.33.x (sqlx 0.8, shares the sea-orm pool via
  `get_postgres_connection_pool`). 0.34-alpha uses sqlx 0.9 — do not use.
- **Queue semantics**: success → `pgmq.delete`; failure → leave the message so the
  visibility timeout retries it. No extra retry loops.
- **CI gate**: `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings` must pass.

## Commits

- Format: `<type>(scope): summary` — scope is usually the crate name
  (e.g. `feat(malprobe-worker): add scan worker skeleton`).
- Keep the summary as short as possible.
- Omit the description body unless the commit is very large.

## Commands

```bash
cargo build --workspace
cargo run -p migration -- generate <name>   # needs DATABASE_URL env
docker compose up -d --build
```
