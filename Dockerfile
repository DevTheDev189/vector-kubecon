# Stage 1: Build Frontend PWA
FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend-pwa
COPY frontend-pwa/package*.json ./
RUN npm ci
COPY frontend-pwa/ ./
RUN npm run build

# Stage 2: Build Rust Binaries
FROM rust:bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend-api/Cargo.toml backend-api/
COPY crowd-simulator/Cargo.toml crowd-simulator/

# Create dummy source trees to cache dependencies
RUN mkdir -p backend-api/src crowd-simulator/src && \
    echo "fn main() {}" > backend-api/src/main.rs && \
    echo "fn main() {}" > crowd-simulator/src/main.rs && \
    cargo build --release && \
    rm -rf backend-api/src crowd-simulator/src

# Copy actual source code
COPY backend-api/src backend-api/src
COPY crowd-simulator/src crowd-simulator/src
RUN touch backend-api/src/main.rs crowd-simulator/src/main.rs && \
    cargo build --release

# Stage 3: Minimal Production Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=rust-builder /app/target/release/backend-api /app/backend-api
COPY --from=rust-builder /app/target/release/crowd-simulator /app/crowd-simulator
COPY --from=frontend-builder /app/frontend-pwa/dist /app/frontend-pwa/dist
COPY seed/ /app/seed/

ENV PORT=8080
ENV RUST_LOG=info,backend_api=debug
ENV STATIC_DIR=/app/frontend-pwa/dist

EXPOSE 8080

ENTRYPOINT ["/app/backend-api"]
