# syntax=docker/dockerfile:1.6

# Base stage with build dependencies required for Bevy/wgpu
FROM rust:1.90-slim-bookworm AS base

ENV CARGO_TERM_COLOR=always \
    RUST_BACKTRACE=1

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    pkg-config \
    build-essential \
    libssl-dev \
    libudev-dev \
    libasound2-dev \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libegl1-mesa-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxcb-render0-dev \
    libxxf86vm-dev \
    libvulkan1 \
    libvulkan-dev \
    mesa-vulkan-drivers \
    vulkan-tools \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Development image keeps the toolchain and source mounted from the host.
FROM base AS dev

CMD ["cargo", "run"]

# Build stage compiles the release binary.
FROM base AS build

# Caching optimisation: copy manifests first
COPY Cargo.toml Cargo.lock ./

# Create placeholder source directory so cargo metadata resolves
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo fetch \
    && rm -rf src

# Copy actual project sources
COPY src ./src
COPY config ./config
COPY README.md ./README.md

RUN cargo build --release --bin thegame

# Runtime image contains only the compiled binary and runtime libs.
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    libudev1 \
    libasound2 \
    libx11-6 \
    libxcursor1 \
    libxrandr2 \
    libxi6 \
    libgl1 \
    libegl1 \
    libwayland-client0 \
    libxkbcommon0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libxcb-render0 \
    libxxf86vm1 \
    libvulkan1 \
    mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=build /workspace/target/release/thegame /usr/local/bin/thegame
COPY --from=build /workspace/config ./config

ENV RUST_BACKTRACE=1

CMD ["/usr/local/bin/thegame"]
