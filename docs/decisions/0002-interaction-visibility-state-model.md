# ADR 0002: Separate interaction and visibility controls

- Status: Accepted
- Date: 2026-11-07

## Context

The HUD must remain visible without blocking underlying applications, become
interactive on demand, and sometimes disappear completely without terminating
its sessions. A simple Active/Passive toggle cannot express temporary hiding.

## Decision

HudSwitch exposes Active, Passive, and Hidden states.

- `Win+Backtick` controls interaction: Passive and Active toggle; Hidden or an
  absent target becomes Active.
- `Win+Shift+Backtick` controls visibility: Active or Passive becomes Hidden,
  remembering its prior state; Hidden restores that state; an absent target is
  launched Passive.
- Prompt-aware Escape changes Active to Passive.
- Unexpected Active focus loss changes Active to Passive.

Hidden remembers the state from which it was entered. Active does not remember
whether it was entered from Hidden.

## Consequences

- The visibility shortcut is reversible: hide then show restores the prior
  interaction state.
- A hidden terminal can be summoned ready for input with one interaction
  shortcut.
- Showing a HUD hidden from Active intentionally restores focus to it.
- The implementation must store a Hidden return state and serialize concurrent
  transitions.

