ARG FEATURES=

FROM rust:1.89 AS builder
ARG FEATURES
ARG TARGETARCH
WORKDIR /usr/src/autokuma

COPY . .
RUN --mount=type=cache,target=/cache/$TARGETARCH,id=cache-ghcr.io/bigboot/autokuma-${FEATURES} \
    cargo install --features "${FEATURES}" --locked --target-dir /cache/$TARGETARCH --path ./autokuma 
 
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY /usr/local/share/ca-certificates/ttd-root-ca.crt /usr/local/share/ca-certificates/ttd-root-ca.crt
RUN update-ca-certificates

COPY --from=builder /usr/local/cargo/bin/autokuma /usr/local/bin/autokuma

ENV AUTOKUMA_DOCKER=1
CMD ["autokuma"]