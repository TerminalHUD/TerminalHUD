# Open product questions

There are currently no unresolved product decisions blocking the MVP.

The settled MVP is:

- Product name: `TerminalHUD`
- Language: Rust unless demonstrated infeasible
- Platform: Windows 11
- Terminal: a dedicated window in the installed Windows Terminal (owner PID may be shared)
- Configuration: validated JSON and CLI helpers
- Runtime UI: native tray icon and menu
- HudSwitch: independent library with a portable core and Win32 backend
- Target count: one
- Display: primary monitor

Technical unknowns that require experiments are tracked in
`validation-plan.md`. Ideas intentionally outside the MVP are tracked in
`future-work.md`.

New entries belong here only when they require a product choice rather than an
engineering experiment.

