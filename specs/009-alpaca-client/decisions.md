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
