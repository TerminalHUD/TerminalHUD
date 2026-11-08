# ADR 0003: Implement Escape through shell line editors

- Status: Accepted
- Date: 2026-11-07

## Context

Escape should make an Active HUD Passive at the shell prompt without preventing
Neovim and other interactive terminal applications from receiving Escape.
Windows Terminal receives the key in both situations and forwards terminal input
through ConPTY, so an external keyboard hook cannot reliably determine whether a
terminal application consumed it.

## Decision

Supported shell line editors implement prompt-aware Escape. They request the
running TerminalHUD instance to become Passive through same-user local IPC.

The default binding triggers only at an empty, idle prompt with no completion or
similar interactive menu. Initial integrations target PowerShell/PSReadLine, WSL
Bash, and Git Bash.

`Win+Backtick` remains the unconditional fallback.

## Consequences

- Escape reaches Neovim and other interactive applications normally.
- Each supported shell requires a small integration and shell-specific tests.
- Integrations must be installed explicitly and preserve user configuration.
- Unsupported shells still have complete HUD control through global shortcuts.

