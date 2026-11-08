# ADR 0004: Install HUD profiles through Windows Terminal fragments

- Status: Accepted
- Date: 2026-11-07

## Context

TerminalHUD needs dedicated profile variants with different focused and
unfocused opacity. Directly rewriting the user's main Windows Terminal
`settings.json` risks damaging comments, formatting, custom settings, or future
schema changes. Modifying ordinary profiles would also affect terminals outside
the HUD.

## Decision

TerminalHUD creates separate HUD variants for discoverable, launchable profiles
and installs them through Windows Terminal JSON fragment extensions under:

```text
%LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\TerminalHUD\
```

Ordinary profiles and the main `settings.json` remain untouched.

## Consequences

- Installation and removal are isolated and reversible.
- TerminalHUD must discover and reproduce relevant launch behavior for explicit,
  dynamic, WSL, and third-party profiles.
- Unsupported or ambiguous profiles require a safe fallback or user selection.
- The adapter must test fragment reload behavior across supported Terminal
  versions.

