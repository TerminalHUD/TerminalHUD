# TerminalHUD

TerminalHUD is a cross-platform utility that turns a terminal window into a
persistent heads-up display. The terminal can remain visible without blocking
mouse or keyboard input, become interactive on demand, and be hidden without
terminating its sessions.

The product is built on **HudSwitch**, a cross-platform, application-agnostic
library for managing active, passive, and hidden states of a window. HudSwitch
has a portable state-machine core and platform-specific window-system backends.
It is a separate library from the beginning so that terminal- and
platform-specific behavior cannot leak into its core API.

The MVP targets Windows 11 and Windows Terminal and is implemented in Rust.
Linux and macOS are future platform milestones. An embedded terminal is a
possible future feature, not part of the MVP.

The repository contains the product/design documentation and the Rust workspace
foundation. Runtime HUD behavior is not implemented yet. See the
[build instructions](docs/building.md) for the crate map and Windows test path.

## License

TerminalHUD is licensed under the [MIT License](LICENSE).

## Core interaction on Windows

- `Win+Backtick` toggles interaction between Active and Passive. If the HUD is
  hidden or not running, it is shown or launched Active.
- `Win+Shift+Backtick` hides the HUD or restores the state from which it was
  hidden. If the HUD is not running, it is launched Passive.
- `Escape` at an empty, idle supported-shell prompt changes Active to Passive.
- `Escape` inside Neovim and other interactive terminal programs is delivered
  normally.

Other platforms will use configurable, platform-appropriate equivalents while
preserving the same state transitions.

See [requirements](docs/requirements.md) and the
[state machine](docs/state-machine.md) for the complete behavior.

## Documentation

- [Contributing](CONTRIBUTING.md) and [development workflow](docs/development.md)
- [Requirements](docs/requirements.md)
- [State machine](docs/state-machine.md)
- [Architecture](docs/architecture.md)
- [Platform support](docs/platform-support.md)
- [Rust application-shell evaluation](docs/ui-toolkit-evaluation.md)
- [Windows Terminal integration](docs/terminal-integration.md)
- [Shell integration](docs/shell-integration.md)
- [Configuration](docs/configuration.md)
- [Open questions](docs/open-questions.md)
- [Validation plan](docs/validation-plan.md)
- [Future work](docs/future-work.md)
- [Architecture decisions](docs/decisions/)

