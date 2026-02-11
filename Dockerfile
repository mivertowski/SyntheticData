# =============================================================================
# Stage 1: Chef - dependency recipe computation
# =============================================================================
FROM rust:1.88-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# =============================================================================
# Stage 2: Planner - compute recipe.json for dependency caching
# =============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: Builder - build with cached dependencies
# =============================================================================
FROM chef AS builder

# Install protoc for gRPC code generation and fontconfig for plotters
RUN apt-get update && apt-get install -y protobuf-compiler libfontconfig1-dev && rm -rf /var/lib/apt/lists/*

# Cook dependencies first (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release -p datasynth-server -p datasynth-cli

# Stage shared libs to a well-known path for the final image
RUN mkdir -p /staging/lib && \
    ARCH=$(uname -m) && \
    cp /usr/lib/${ARCH}-linux-gnu/liblzma.so.5* /staging/lib/

# =============================================================================
# Stage 4: Runtime - minimal distroless image
# =============================================================================
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/datasynth-server /usr/local/bin/datasynth-server
COPY --from=builder /app/target/release/datasynth-data /usr/local/bin/datasynth-data

# Copy shared libraries needed for Parquet (lzma)
COPY --from=builder /staging/lib/ /usr/lib/

USER nonroot:nonroot

EXPOSE 50051 3000

ENTRYPOINT ["datasynth-server"]
CMD ["--host", "0.0.0.0", "--port", "50051", "--rest-port", "3000"]
