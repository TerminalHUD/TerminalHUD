# ADR 0006: Slint settings UI proposal

- Status: Superseded by ADR 0007
- Date: 2026-11-07

## Context

TerminalHUD and HudSwitch need safe long-running event handling, explicit native
API boundaries, cross-platform backends, a tray application, and a relatively
small settings UI. The implementation language and UI toolkit must not couple
HudSwitch to a product framework.

## Original proposal

Use Rust for the project and Slint for the TerminalHUD application shell,
provided a focused Windows prototype passes the acceptance criteria in
`docs/ui-toolkit-evaluation.md`.

Keep Slint out of HudSwitch, terminal adapters, and TerminalHUD core. Those
crates communicate with the app shell through toolkit-neutral commands, events,
and data models.

Use Tauri 2 as the comparison/fallback if Slint's relatively new tray support,
event-loop integration, or packaging creates material problems.

## Superseding context

The first release does not require a graphical configuration window. A native
tray menu, validated JSON configuration, and CLI commands meet the product
requirements with substantially less machinery. Slint and Tauri are therefore
removed from the critical path and can be reconsidered if a GUI becomes useful.

## Historical consequences considered

- The repository becomes a Cargo workspace.
- Native API use is isolated in platform crates and explicit `unsafe` sections.
- TerminalHUD gains a declarative cross-platform UI without a web frontend.
- Packaging and startup integration remain platform responsibilities rather
  than framework plugins.
- The project must choose a Slint license path and satisfy its attribution or
  copyleft/commercial requirements.
- The decision remains reversible because no core crate depends on Slint.

