# Caddy2 Integration Example

This example demonstrates how to use AutoKuma with Caddy2 to automatically create monitors for all your Caddy-hosted websites.

## How It Works

AutoKuma's Caddy integration works by:

1. **Polling Caddy's Admin API**: AutoKuma fetches the current configuration from Caddy's admin API endpoint (default: `http://localhost:2019/config/`)
2. **Parsing JSON Structure**: The JSON response is parsed to navigate through `apps.http.servers` → `routes` → `match` → `host` arrays
3. **Extracting Hosts**: All unique hostnames are extracted from route matchers
4. **Wildcard Processing**: Wildcard prefixes (e.g., `*.example.com`) are stripped to become `example.com`
5. **Monitor Creation**: For each host, an HTTP/HTTPS monitor is created with the format:
   - Monitor ID: `caddy/{hostname}` (e.g., `caddy/example.com`)
   - Monitor Name: `{prefix}{hostname}` (prefix is optional)
   - URL: `https://{hostname}` or `http://{hostname}`
6. **Automatic Sync**: On each AutoKuma sync interval, the process repeats to detect new/removed hosts

### Architecture

```
┌─────────────┐          ┌──────────────┐         ┌──────────────┐
│   Caddy2    │  HTTP    │   AutoKuma   │  API    │ Uptime Kuma  │
│  Admin API  │◄─────────┤ Caddy Source │────────►│   Monitors   │
└─────────────┘  GET     └──────────────┘ Socket  └──────────────┘
                 /config/                   IO
```

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
      AUTOKUMA__CADDY__PARENT_NAME: "caddy-group"  # Optional: organize in a group
      
      # Static monitors path for creating the group
      AUTOKUMA__STATIC_MONITORS: "/data/monitors"
    volumes:
      - autokuma-data:/data
      - ./monitors:/data/monitors  # Mount folder with static monitor definitions

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

4. **Optional: Create a group for organizing monitors**

Create a file `./monitors/caddy-group.json`:

```json
{
    "name": "Caddy Services",
    "type": "group"
}
```

This creates a group folder in Uptime Kuma where all Caddy monitors will be organized.

## What Happens

When AutoKuma starts with Caddy integration enabled:

1. AutoKuma fetches the Caddy configuration from `http://caddy:2019/config/`
2. It extracts all host names: `example.com`, `www.example.com`, `api.example.com`
3. It creates HTTP/HTTPS monitors in Uptime Kuma:
   - `Web - example.com` → `https://example.com` (under "Caddy Services" group)
   - `Web - www.example.com` → `https://www.example.com` (under "Caddy Services" group)
   - `Web - api.example.com` → `https://api.example.com` (under "Caddy Services" group)

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
| `AUTOKUMA__CADDY__PARENT_NAME` | (none) | Optional parent group to organize monitors |

## Organizing with Groups

To keep your Caddy monitors organized in Uptime Kuma:

1. **Create the group definition** in a static monitor file (e.g., `./monitors/caddy-group.json`):
```json
{
    "name": "Caddy Services",
    "type": "group"
}
```

2. **Set the parent_name** configuration to match the group's **autokuma ID** (filename without extension):
```yaml
AUTOKUMA__CADDY__PARENT_NAME: "caddy-group"
```

**Important**: The `parent_name` must be the **autokuma ID**, NOT the display name shown in Uptime Kuma.

- The autokuma ID is the filename without extension (e.g., `caddy-group` from `caddy-group.json`)
- The display name is what you see in Uptime Kuma UI (e.g., "Caddy Services")

**Example**:
- ✅ Correct: `AUTOKUMA__CADDY__PARENT_NAME: "caddy-group"` (matches filename `caddy-group.json`)
- ❌ Wrong: `AUTOKUMA__CADDY__PARENT_NAME: "Caddy Services"` (this is display name, will fail with "No monitor named Caddy Services could be found")

If you have an existing group in Uptime Kuma, you need to create a corresponding static monitor definition file with the autokuma ID that matches what you want to use as `parent_name`.

All Caddy monitors will now appear nested under the "Caddy Services" folder in Uptime Kuma's UI.

## Wildcard Hosts

If your Caddyfile contains wildcard hosts like `*.example.com`, AutoKuma will:
- Strip the `*.` prefix
- Create a monitor for `example.com`

## Troubleshooting

### Error: "No monitor named X could be found"

This error occurs when the `parent_name` configuration doesn't match an existing autokuma ID.

**Common causes**:
1. **Using display name instead of autokuma ID**: The `parent_name` must be the autokuma ID (filename without extension), not the display name shown in Uptime Kuma.
   - Wrong: `AUTOKUMA__CADDY__PARENT_NAME: "Services"` (display name)
   - Correct: `AUTOKUMA__CADDY__PARENT_NAME: "services-group"` (autokuma ID from `services-group.json`)

2. **Group doesn't exist as a static monitor**: If you have a group in Uptime Kuma but no corresponding static monitor definition, AutoKuma can't find it. Create a static monitor file like `./monitors/services-group.json`:
```json
{
    "name": "Services",
    "type": "group"
}
```

3. **Mismatched autokuma ID**: Make sure the `parent_name` value exactly matches the filename (without `.json` extension) of your group definition.

**Enable debug logging** to see detailed information:
```yaml
environment:
  RUST_LOG: "debug"
```

This will show logs like:
```
[autokuma::sources::caddy_source] DEBUG: Caddy monitors will be organized under parent group with autokuma ID: 'caddy-group'
[autokuma::sources::caddy_source] DEBUG: Setting parent_name='caddy-group' for monitor 'example.com'
```

## Security Considerations

- The Caddy admin API should only be accessible within your Docker network
- Don't expose port 2019 to the public internet
- Consider using authentication for the Caddy admin API in production
