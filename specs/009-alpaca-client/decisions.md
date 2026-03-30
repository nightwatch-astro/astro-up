# Decisions Report: 009-alpaca-client
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **Use ascom-alpaca-core crate**: Already developed in the nightwatch-astro org. Provides typed API request/response structs.
- **Alpaca takes precedence over registry**: The Alpaca API reports the actual loaded driver version, which is more authoritative than what's in the registry (which may be stale after updates).
- **mDNS discovery with fallback to configured host**: mDNS is the Alpaca standard but may not work on all networks. Allow manual host:port config.
- **5-second discovery timeout**: Alpaca servers should respond quickly on LAN. 5s is generous enough for slow networks.

## Questions I Would Have Asked
- Q1: Should we support remote Alpaca servers (not localhost)? Decision: Yes — some setups have the ASCOM server on a separate machine. Configurable host.
- Q2: Should we cache Alpaca discovery results? Decision: No — device state changes frequently. Always query fresh.

## Clarify-Phase Decisions

### C1: UDP broadcast discovery on port 32227
**Decision**: This is the Alpaca standard. Send broadcast, collect responses for 2s, use the first responder. If no response, fall back to configured host.

### C2: 2s discovery + 5s API timeout
**Decision**: LAN operations should be near-instant. 2s for discovery catches slow networks. 5s for API calls handles loaded ASCOM servers. Both are configurable via spec 004.

### C3: Query all configured devices, not just connected ones
**Decision**: Report all devices from `/configureddevices` with their connection state. This gives users a full picture even when hardware is disconnected.

### C4: Alpaca client lives in astro-up-core, not a separate crate
**Decision**: The Alpaca client depends on ascom-alpaca-core for types but the client logic belongs in astro-up-core. It's not reusable enough to warrant a separate crate.

### C5: No persistent Alpaca connection
**Decision**: Each scan creates a fresh HTTP client, queries, and tears down. No long-lived connection to the Alpaca server. Astrophotography sessions are long-running — we shouldn't hold resources between scans.
