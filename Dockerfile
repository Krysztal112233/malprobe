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
