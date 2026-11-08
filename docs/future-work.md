# Future work

These ideas are intentionally outside the Windows MVP.

## Platform support

- Linux/X11 backend
- Linux/Wayland research and capability model
- macOS backend and Accessibility-permission UX
- Platform-specific tray and packaging validation
- Non-Windows terminal adapters

No Wayland behavior is promised yet. Future research can choose cooperation,
an owned layer-shell surface, compositor integration, or reduced functionality.

## Terminal hosting

- Cooperative terminal adapters that apply HUD behavior inside the
  window-owning process
- Evaluation of embeddable terminal engines
- Optional TerminalHUD-owned terminal window
- Terminal rendering features such as outlined or shadowed glyphs

An embedded terminal is a possible feature, not a committed requirement. An
in-house terminal emulator should not be attempted before evaluating reusable
engines.

## Product expansion

- Multiple simultaneous HUD targets
- Secondary-monitor selection
- General-purpose HudSwitch host application
- Graphical configuration editor backed by the same JSON model
- Publishing HudSwitch as a standalone Rust crate
- Additional shell integrations

