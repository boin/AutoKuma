# Caddy2 Integration Example

This example demonstrates how to use AutoKuma with Caddy2 to automatically create monitors for all your Caddy-hosted websites.

## Setup

1. **Configure Caddy** to expose its admin API (enabled by default on port 2019)

2. **Configure AutoKuma** with environment variables:

```yaml
services:
  autokuma:
    image: ghcr.io/bigboot/autokuma:latest
    restart: unless-stopped
    environment:
      # Uptime Kuma connection
      AUTOKUMA__KUMA__URL: http://uptime-kuma:3001
      AUTOKUMA__KUMA__USERNAME: admin
      AUTOKUMA__KUMA__PASSWORD: your_password
      
      # Enable Caddy integration
      AUTOKUMA__CADDY__ENABLED: "true"
      AUTOKUMA__CADDY__URL: "http://caddy:2019/config/"
      AUTOKUMA__CADDY__USE_HTTPS: "true"
      AUTOKUMA__CADDY__MONITOR_NAME_PREFIX: "Web - "
    volumes:
      - autokuma-data:/data

  caddy:
    image: caddy:latest
    ports:
      - "80:80"
      - "443:443"
      - "2019:2019"  # Admin API
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config

  uptime-kuma:
    image: louislam/uptime-kuma:latest
    ports:
      - "3001:3001"
    volumes:
      - uptime-kuma-data:/app/data

volumes:
  autokuma-data:
  caddy_data:
  caddy_config:
  uptime-kuma-data:
```

3. **Example Caddyfile**:

```
example.com {
    reverse_proxy backend:8080
}

www.example.com {
    reverse_proxy backend:8080
}

api.example.com {
    reverse_proxy api:3000
}
```

## What Happens

When AutoKuma starts with Caddy integration enabled:

1. AutoKuma fetches the Caddy configuration from `http://caddy:2019/config/`
2. It extracts all host names: `example.com`, `www.example.com`, `api.example.com`
3. It creates HTTP/HTTPS monitors in Uptime Kuma:
   - `Web - example.com` → `https://example.com`
   - `Web - www.example.com` → `https://www.example.com`
   - `Web - api.example.com` → `https://api.example.com`

## Dynamic Updates

AutoKuma will automatically:
- Add monitors when new hosts are added to Caddy
- Remove monitors when hosts are removed from Caddy
- Keep monitors in sync with your Caddy configuration

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `AUTOKUMA__CADDY__ENABLED` | `false` | Enable/disable Caddy integration |
| `AUTOKUMA__CADDY__URL` | `http://localhost:2019/config/` | Caddy admin API endpoint |
| `AUTOKUMA__CADDY__USE_HTTPS` | `true` | Create HTTPS monitors (vs HTTP) |
| `AUTOKUMA__CADDY__MONITOR_NAME_PREFIX` | (none) | Optional prefix for monitor names |

## Wildcard Hosts

If your Caddyfile contains wildcard hosts like `*.example.com`, AutoKuma will:
- Strip the `*.` prefix
- Create a monitor for `example.com`

## Security Considerations

- The Caddy admin API should only be accessible within your Docker network
- Don't expose port 2019 to the public internet
- Consider using authentication for the Caddy admin API in production
