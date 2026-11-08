# ADR 0009: Create every HUD Active

- Status: Accepted
- Date: 2026-11-08

## Context

Controlled Windows Terminal startup experiments observed target activation
despite documented no-activation show commands. A windowed control also
activated, so fullscreen is not necessary to reproduce the behavior. Its
internal cause is unproven. Requiring Passive creation would require further
startup workarounds and lifecycle complexity.

The user chose Active creation for every fresh HUD, including both shortcuts
when no target exists and relaunch after the terminal closes.

## Decision

TerminalHUD creates every new target Active. Initial state is fixed for the MVP;
it is not a configurable Passive or Hidden launch option. Startup intentionally
takes focus. Existing targets retain Active, Passive, and Hidden transitions,
including remembered Hidden state and normal focus restoration.

TerminalHUD selects the creation state through a generic HudSwitch controller
policy applied after target validation. HudSwitch's default requested-state
behavior and reusable Passive creation contract remain available to other
consumers. The adapter must honor its requested creation intent exactly.

## Consequences

- Both shortcuts create Active when the target is absent or becomes invalid.
- An absent passive IPC request remains a no-op; it does not create a target.
- Hidden-start workarounds and no-focus creation experiments are unnecessary
  for this MVP path.
- Passive click-through, rendering, focus restoration, session preservation,
  and safe lifecycle handling still require real Windows evidence.
- Monorepo structure and core/backend/adapter boundaries remain unchanged.
