ARG FEATURES=

FROM rust:1.89 AS builder
ARG FEATURES
ARG TARGETARCH
WORKDIR /usr/src/autokuma
COPY . .
RUN --mount=type=cache,target=/cache/$TARGETARCH,id=cache-ghcr.io/bigboot/autokuma-${FEATURES} \
    cargo install --features "${FEATURES}" --locked --target-dir /cache/$TARGETARCH --path ./autokuma && \
    cargo install --features "${FEATURES}" --locked --target-dir /cache/$TARGETARCH --path ./kuma-cli
 
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/autokuma /usr/local/bin/autokuma
COPY --from=builder /usr/local/cargo/bin/kuma /usr/local/bin/kuma

ENV AUTOKUMA_DOCKER=1
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -sf http://localhost:8090/health || exit 1
CMD ["autokuma"]