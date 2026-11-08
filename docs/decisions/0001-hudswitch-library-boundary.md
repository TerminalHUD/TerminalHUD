# ADR 0001: HudSwitch is an independent library

- Status: Accepted
- Date: 2026-11-07

## Context

TerminalHUD needs generic window behavior: launch and acquire a window, make it
topmost, switch it between interactive and click-through modes, hide and restore
it, and manage focus. These capabilities are useful outside terminals and need
platform-specific implementations.

If this behavior begins inside the TerminalHUD executable, terminal-specific
exceptions are likely to leak into the state machine and make later extraction
difficult.

## Decision

HudSwitch is a separate cross-platform library from the first implementation.
It consists of a portable state-machine/contracts core and platform backends.
TerminalHUD and terminal adapters depend on HudSwitch. HudSwitch does not depend
on product projects and contains no Terminal-specific knowledge.

The projects initially live in one monorepo. HudSwitch may later be published
independently after its API has been exercised by the product.

## Consequences

- The architectural boundary is testable and enforceable.
- HudSwitch requires a generic test-window application.
- Native platform types and behavior are isolated in backend packages.
- Some apparently convenient Terminal-specific shortcuts must remain in the
  adapter, even when that requires more explicit interfaces.
- Monorepo changes can update the library, adapter, product, tests, and installer
  atomically.

