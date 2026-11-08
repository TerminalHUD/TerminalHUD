# ADR 0007: Use JSON configuration and a native tray for v1

- Status: Accepted
- Date: 2026-11-07

## Context

TerminalHUD needs a small always-running control surface, but its configuration
does not require a graphical editor. Carrying a full GUI toolkit or webview only
for occasional settings changes would add runtime, packaging, licensing, and
event-loop complexity.

## Decision

The first release uses:

- a documented, validated per-user JSON configuration file;
- CLI commands to locate, open, validate, and reload configuration;
- CLI commands to preview, install, and remove shell integrations; and
- a native tray icon and menu for common runtime actions.

No graphical settings window or general UI toolkit is required. The current
Rust direction is a lightweight tray crate such as `tray-icon` combined with a
minimal cross-platform or backend-native event loop. The prototype determines
the final event-loop composition.

## Consequences

- Slint, Tauri, and webview dependencies are unnecessary in v1.
- JSON remains the single source of truth if a graphical editor is added later.
- Configuration validation and actionable error reporting become important user
  experience features.
- Shortcut recording and opacity previews are unavailable initially; users edit
  explicit values and reload.
- The tray and backend event loops require a focused integration prototype.

