FROM node:25-bookworm AS frontend

WORKDIR /app
COPY package.json yarn.lock ./
COPY apps/desktop/package.json ./apps/desktop/package.json
RUN yarn install --frozen-lockfile
COPY apps/desktop ./apps/desktop
WORKDIR /app/apps/desktop
RUN yarn lint
RUN yarn typecheck
RUN yarn test
RUN yarn build

FROM rust:1.91-bookworm AS rust

WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libayatana-appindicator3-dev \
        libgtk-3-dev \
        librsvg2-dev \
        libssl-dev \
        libwebkit2gtk-4.1-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/desktop/src-tauri ./apps/desktop/src-tauri
RUN cargo fmt --all -- --check
RUN cargo clippy --workspace --all-targets -- -D warnings
RUN cargo test --workspace

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=frontend /app/apps/desktop /app/apps/desktop
COPY --from=rust /app /app
CMD ["bash", "-lc", "echo DevAtlas desktop validation image"]
