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

Run `python3 scripts/check_repository.py` and `git diff --check` before handoff.
There is no Cargo workspace yet; add build, lint, and test checks when one exists.
See the adoption section in `docs/development.md` for pending setup decisions.
