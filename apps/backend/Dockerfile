## syntax=docker/dockerfile:1.7

# Stage 1: Recipes & Plan
FROM rust:1.89-alpine3.21 AS builder

WORKDIR /app
RUN apk add --no-cache musl-dev pkgconfig openssl-dev libressl-dev

# Use sparse registry for faster updates
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_TARGET_DIR=/target

# Copy only dependency-related files first
COPY Cargo.toml Cargo.lock /app/

# Build dummy project to cache dependencies
RUN mkdir -p /app/src \
    && echo "fn main() {}" > /app/src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release

# Copy real source code
COPY src /app/src
COPY migrations /app/migrations

# Ensure the real main.rs is newer than the dummy one to force rebuild
RUN touch src/main.rs

# Build real project
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release \
    && cp /target/release/council-backend /app/council-backend

# Stage 2: Final Runtime Image
FROM alpine:3.21

WORKDIR /app
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/council-backend /usr/local/bin/council-backend

# Create logs and data directory with broad permissions to support different UIDs
RUN mkdir logs data && chmod 777 logs data

# Set environment variables for runtime
ENV APP_PORT=8080
ENV LOG_LEVEL=info

EXPOSE 8080
CMD ["/usr/local/bin/council-backend"]
