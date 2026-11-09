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
- Unsupported or ambiguous profiles require explicit resolution or user selection;
  never guess another shell.
- The adapter must test fragment reload behavior across supported Terminal
  versions.


## Installed-settings limitation

User profile defaults outrank a new fragment profile in Terminal's native loader.
Consequently a fragment cannot always preserve a selected profile's inline
settings or apply HUD opacity. [ADR 0011](0011-owned-profile-entries.md) proposes
byte-preserving, receipt-owned user-list entries after explicit setup consent.
That proposal does not authorize a main-settings write by itself.
