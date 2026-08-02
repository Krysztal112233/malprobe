# malprobe-web

Vue 3 + Vite + TypeScript frontend for malprobe.

## Stack

- Vue 3 + Vue Router
- Naive UI (dark theme)
- TanStack Query — server state + polling (`pending → scanning → completed`)
- Vite dev-server proxy forwards `/files`, `/docs` to the backend
  (`MALPROBE_BACKEND`, default `http://127.0.0.1:8000`), so no CORS config
  is needed during development.

Note: frontend routes live under `/report/:id` and `/hash/:sha256` — they must
not collide with backend API paths (`/files*`), which are proxied to the
backend both in dev (Vite proxy) and prod (reverse proxy).

## Backend address

- **Dev**: `MALPROBE_BACKEND` (default `http://127.0.0.1:8000`) sets the
  Vite proxy target — no CORS needed.
- **Build**: `VITE_API_BASE_URL` (see `.env.example`) bakes an API origin
  into the bundle, e.g. `VITE_API_BASE_URL=http://192.168.1.10:8000 pnpm build`.
  Empty = same-origin requests; cross-origin bases require CORS on the
  backend (`cors.allow_origins` in `malprobe.toml`).

## Commands

```bash
pnpm install
pnpm dev      # dev server on :5173, proxies API to the backend
pnpm build    # vue-tsc type check + production bundle
```
