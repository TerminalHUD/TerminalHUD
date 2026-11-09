# ADR 0008: Use Rust for a Windows Terminal MVP

- Status: Accepted
- Date: 2026-11-08

## Context

The product behavior is sufficiently defined to begin a focused MVP. Continuing
to hold the implementation language and first platform open would delay useful
feasibility work without reducing meaningful risk.

## Decision

- The product name is `TerminalHUD`.
- Implement TerminalHUD and HudSwitch in Rust using a Cargo workspace.
- Target Windows 11 and Windows Terminal for the MVP.
- Originally required a dedicated separate Windows Terminal process. This clause
  is superseded by [ADR 0010](0010-installed-terminal-window.md): use the installed
  Terminal with a dedicated HUD window and allow its owner process to be shared.
- Treat embedded terminal hosting as a possible future feature, not an MVP
  requirement.
- Do not require Linux, macOS, X11, or Wayland support for the MVP.

Rust is reconsidered only if the feasibility prototype identifies a fundamental
blocker that cannot reasonably be isolated behind a small native boundary.

## Consequences

- MVP planning can optimize for a working Windows experience.
- Portable HudSwitch contracts remain valuable, but speculative non-Windows
  abstractions must not slow the Windows implementation.
- The repository uses Cargo workspace boundaries to preserve separation between
  core, Win32 backend, TerminalHUD, and Windows Terminal adapter.
- Wayland and embedded-terminal research moves to future work.

