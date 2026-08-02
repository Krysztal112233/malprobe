FROM docker.io/library/rust:slim-trixie AS backend-builder
WORKDIR /builder
RUN apt update && apt install build-essential curl wget file libssl-dev pkg-config -y
COPY . .
RUN cargo build --all -r

FROM docker.io/library/debian:trixie-slim AS backend
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /builder/target/release/malprobe /app/
COPY malprobe.toml /app/
CMD [ "./malprobe" ]

FROM docker.io/library/debian:trixie-slim AS worker
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /builder/target/release/malprobe-worker /app/
COPY malprobe.toml /app/
CMD [ "./malprobe-worker" ]

FROM docker.io/library/debian:trixie-slim AS migration
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /builder/target/release/migration /app/
# Runs all pending migrations against DATABASE_URL and exits.
# compose starts it once (restart: "no") and gates the backend/worker
# on `service_completed_successfully`.
CMD [ "./migration" ]

# Prebaked ClamAV image based on ghcr.io/extremeshok/clamav-unofficial-sigs.
# Official ClamAV databases (freshclam) and unofficial signature databases
# (clamav-unofficial-sigs: Sanesecurity, URLhaus, Linux Malware Detect, ...)
# are downloaded once during the build and baked into the image.
# At runtime only clamd is started; freshclam and the unofficial-sigs update
# loop are disabled, so the container never updates its databases online.
FROM ghcr.io/extremeshok/clamav-unofficial-sigs:latest AS clamav

RUN freshclam --foreground --stdout && \
    rm -f /var/lib/clamav/freshclam.dat /var/lib/clamav/mirrors.dat

RUN printf '%s\n' \
        'user_configuration_complete="yes"' \
        'enable_random="no"' \
    > /etc/clamav-unofficial-sigs/user.conf && \
    clamav-unofficial-sigs.sh

ENV CLAMAV_NO_FRESHCLAMD=true

ENTRYPOINT ["/init"]

HEALTHCHECK --interval=1m --timeout=30s --start-period=6m --retries=3 \
    CMD ["/usr/local/bin/clamdcheck.sh"]

# Build the Vue frontend inside the web/ directory.
FROM docker.io/library/node:22-alpine AS web-builder
WORKDIR /app
COPY . .
WORKDIR /app/web
# API origin baked into the bundle; the containerised NGINX expects /api,
# which it proxies to the malprobe service (see web/nginx.conf).
ARG VITE_API_BASE_URL=/api
ENV VITE_API_BASE_URL=$VITE_API_BASE_URL
RUN npm install -g pnpm@11 && \
    pnpm install --frozen-lockfile && \
    pnpm build

# Serve the built frontend with NGINX and proxy /api -> malprobe:8000.
FROM docker.io/library/nginx:alpine AS web
COPY web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=web-builder /app/web/dist /usr/share/nginx/html
