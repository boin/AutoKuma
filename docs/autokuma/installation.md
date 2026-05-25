---
icon: lucide/package-open
---

# Installation

## Docker (Recommended)

AutoKuma is available as a Docker image on the [GitHub Container Registry](https://github.com/BigBoot/AutoKuma/pkgs/container/autokuma).

```bash
# For Uptime Kuma V2
docker pull ghcr.io/bigboot/autokuma:latest

# For Uptime Kuma V1
docker pull ghcr.io/bigboot/autokuma:uptime-kuma-v1-latest
```

!!! warning "Dev builds"
    Development builds (`master` tag) may contain breaking changes without warning. Do not use them in production.

    ```bash
    docker pull ghcr.io/bigboot/autokuma:master
    ```

## Docker Compose

Here is a minimal `docker-compose.yml` to get AutoKuma running alongside Uptime Kuma:

```yaml
version: '3'

services:
  autokuma:
    image: ghcr.io/bigboot/autokuma:latest
    restart: unless-stopped
    environment:
      AUTOKUMA__KUMA__URL: http://uptime-kuma:3001
      # AUTOKUMA__KUMA__USERNAME: <username>
      # AUTOKUMA__KUMA__PASSWORD: <password>
      # AUTOKUMA__KUMA__MFA_TOKEN: <token>
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - autokuma-data:/data

volumes:
  autokuma-data:
```

## Prebuilt Binaries

Prebuilt binaries for Windows, Linux, and macOS are available on the [GitHub Releases](https://github.com/BigBoot/AutoKuma/releases/latest) page.

## Build from Source

```bash
cargo install --git https://github.com/BigBoot/AutoKuma.git autokuma
```
