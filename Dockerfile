FROM oven/bun:alpine AS web
WORKDIR /app

COPY package.json package-lock.json bun.lock ./
COPY apps/web/package.json apps/web/package.json
RUN bun install --frozen-lockfile

COPY apps/web apps/web
COPY components.json ./
RUN cd apps/web && bun run build

FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY --from=web /app/apps/web/dist apps/web/dist
RUN cargo build --release --locked -p msm-app

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/msm-app /usr/local/bin/msm-app

ENV MSM_BIND_ADDR=0.0.0.0:3000
ENV MSM_DATABASE_URL=sqlite:/data/msm.sqlite3
ENV MSM_ASSET_DIR=/data/assets

VOLUME ["/data"]
EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/msm-app"]
