# ADR 0005: TerminalHUD and HudSwitch are cross-platform

- Status: Accepted
- Date: 2026-11-07

## Context

The HUD interaction model is not inherently tied to Windows Terminal. Other
platforms have terminal applications and window APIs that could provide the same
user experience. TerminalHUD manages only a dedicated terminal it launches, but
that terminal may still own its native window in a separate process. Win32, X11,
Wayland, and macOS expose materially different capabilities for controlling that
separate-process window.

## Decision

TerminalHUD and HudSwitch are cross-platform projects. Windows 11 and Windows
Terminal are the first supported milestone.

HudSwitch separates its portable state machine and contracts from
platform-specific backends. TerminalHUD separates portable orchestration from
terminal adapters and platform UI/packaging. Native window handles do not cross
the portable boundary, and backends report capabilities explicitly.

## Consequences

- Windows-specific choices cannot define HudSwitch's core API.
- Windows progress does not wait for unfinished Linux or macOS backends.
- X11 can likely support an external-window model similar to Windows.
- Wayland may require terminal cooperation, a TerminalHUD-hosted layer-shell
  window, or compositor-specific support; spawning a terminal does not confer
  control over its Wayland surface.
- macOS may require Accessibility permission for cross-process control.
- Windows is the only implementation required for the MVP; non-Windows behavior
  remains future work rather than a release blocker.

