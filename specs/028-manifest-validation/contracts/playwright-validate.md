# Playwright Validation — Interactive via MCP

## Approach

Playwright is used **interactively** during the one-time audit via the MCP Playwright server tools (already available in the development environment). It is NOT a script, NOT embedded in the checker, and NOT part of CI.

## Workflow

For each `browser_scrape` or `html_scrape` manifest:

1. Use `browser_navigate` to load the vendor page URL
2. Use `browser_snapshot` to capture the DOM
3. Apply the manifest's CSS selector (if any) via `browser_evaluate`
4. Apply the manifest's regex to extract the version
5. Compare against the checker's discovered version
6. If they disagree → investigate and fix the manifest

## Anti-Detection

Use `browser_evaluate` to inject stealth JS matching the checker's chromiumoxide stealth:

```javascript
Object.defineProperty(navigator, 'webdriver', { get: () => false });
Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
window.chrome = { runtime: {} };
```

## No CI Integration

The 6-hourly pipeline continues using the chromiumoxide-based `browser_scrape` provider. Playwright is strictly a one-time validation tool for the audit phase.
