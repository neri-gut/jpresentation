FROM mcr.microsoft.com/devcontainers/javascript-node:22-bookworm

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    clang \
    cmake \
    pkg-config \
    libgtk-3-dev \
    && rm -rf /var/lib/apt/lists/*

USER node
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && . "$HOME/.cargo/env" \
    && rustup component add rustfmt clippy rust-analyzer
ENV PATH="/home/node/.cargo/bin:${PATH}"
