# Configuration

Astro-Up uses a layered TOML configuration system with sensible defaults. **Every setting is configurable from the GUI, CLI, or config file** — choose whichever you prefer.

## GUI Settings

Open **Settings** from the sidebar to access all configuration options organized by category: General, Catalog, Network, Backup, Notifications, and Logging.

## Config File

The config file is at:

```
%APPDATA%\nightwatch\astro-up\config.toml
```

It's created automatically on first run. Edit it directly or via:

```sh
astro-up config --edit
```

## CLI Overrides

Most settings can be overridden per-command:

```sh
astro-up --log-level debug scan
astro-up --config C:\path\to\config.toml list
```

## Quick Reference

| Section | Key settings |
|---------|-------------|
| **General** | Font size, install scope, install method, auto-scan, auto-update |
| **Catalog** | Catalog URL, cache TTL |
| **Network** | Proxy, timeouts, speed limit |
| **Backup** | Schedule, retention limits |
| **Notifications** | Enable/disable, display duration, per-type toggles |
| **Logging** | Log level, file logging |

For the complete field reference, see [Configuration File Reference](/reference/config).
