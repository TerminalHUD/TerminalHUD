# Agent instructions

Follow [the contributor workflow](docs/development.md). Before work, read the
project's relevant requirements/design, assignment, and current handoff.
Repository authority and staffing are in `ratpack.json`; task owners and
dependencies are in the tracker.

For TerminalHUD, read [requirements](docs/requirements.md),
[architecture](docs/architecture.md), and the relevant accepted
[architecture decisions](docs/decisions/) before changing product behavior.
Keep HudSwitch core independent of terminal, UI, and OS-specific code. The MVP
uses Rust, Windows 11, and a dedicated Windows Terminal process; other platforms
and an embedded terminal remain future work.

Run the checks in `ratpack.json` before handoff. See [building](docs/building.md)
for the workspace map, portable checks, and native Windows build/test path.
Windows behavior requires native Windows evidence; WSL-only tests are insufficient.
The persistent coordinator handles coordination only; workflow support owns setup
changes, and implementation staffing is agreed when product work is ready.
Read current setup decisions and handoffs in Linear, not repository status notes.
