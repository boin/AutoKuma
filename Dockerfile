ARG FEATURES=

FROM rust:1.89 AS builder
ARG FEATURES
ARG TARGETARCH
WORKDIR /usr/src/autokuma

# Inject Root CA for internal resources
RUN --mount=type=secret,id=root_ca \
    cp /run/secrets/root_ca /usr/local/share/ca-certificates/ttd-root-ca.crt && \
    update-ca-certificates

ENV SSL_CERT_DIR=/etc/ssl/certs
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

COPY . .
RUN --mount=type=cache,target=/cache/$TARGETARCH,id=cache-ghcr.io/bigboot/autokuma-${FEATURES} \
    cargo install --features "${FEATURES}" --locked --target-dir /cache/$TARGETARCH --path ./autokuma 
 
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 注意：Secret 默认挂载到 /run/secrets/ID
RUN --mount=type=secret,id=root_ca \
    cp /run/secrets/root_ca /usr/local/share/ca-certificates/ttd-root-ca.crt && \
    update-ca-certificates

ENV SSL_CERT_DIR=/etc/ssl/certs
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

COPY --from=builder /usr/local/cargo/bin/autokuma /usr/local/bin/autokuma

ENV AUTOKUMA_DOCKER=1
CMD ["autokuma"]