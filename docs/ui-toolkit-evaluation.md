# Application shell evaluation

Status: no general UI toolkit required for v1.

TerminalHUD is primarily a long-running background process. Its required user
interface is a native tray icon and menu; configuration can be a validated JSON
file with CLI helpers. This removes the need to select and carry a graphical UI
framework in the first release.

## Required application-shell capabilities

- Native tray/status icon on Windows, Linux, and macOS
- Native menu items with dynamic enabled/checked state
- Operation with no visible application window
- Integration with HudSwitch backend events
- Same-user IPC and single-instance behavior
- Platform startup registration and packaging
- Low idle overhead

## Recommended direction

Use Rust with a focused tray crate rather than Slint, Tauri, or another complete
GUI framework. `tray-icon` is the leading candidate because it supports Windows,
macOS, and Linux/BSD with native platform mechanisms and menus.

The tray crate requires an appropriate platform event loop. The prototype should
compare:

- `tray-icon` with a small Tao event loop; and
- `tray-icon` integrated directly with the event loop already required by each
  HudSwitch backend.

Prefer the design with one authoritative UI/event thread and explicit channels
from process monitoring and IPC workers. Do not make HudSwitch core depend on
the tray or event-loop crate.

## Why not Slint for v1

Slint is a credible future choice and now provides cross-platform system-tray
support. It is unnecessary when no settings window is required. Deferring it
avoids a renderer, UI DSL, toolkit licensing decision, and a second abstraction
over the native event loop.

## Why not Tauri for v1

Tauri provides mature tray, autostart, global-shortcut, single-instance,
updater, and bundling infrastructure. It also introduces a webview/application
framework whose platform plugins overlap with HudSwitch and TerminalHUD's own
platform responsibilities. That tradeoff is unattractive without a graphical
frontend.

## Why not egui/eframe for v1

egui is useful for rapidly building a settings UI, but it is still a rendering
and window framework. It provides no value when configuration is edited as JSON
and runtime actions fit in a native tray menu.

## Configuration experience

The CLI should make JSON configuration approachable:

```text
terminalhud config path
terminalhud config open
terminalhud config validate
terminalhud reload
```

The tray menu should expose Open, Reload, and Validate Configuration actions.
Validation failures preserve the last known-good configuration and produce an
actionable message with the failing path and property.

## Required prototype

Before accepting the final app-shell crates, build a disposable Windows
prototype that proves:

1. It starts with only a tray icon visible.
2. Tray actions drive a fake HudSwitch controller.
3. `Win+Backtick` and `Win+Shift+Backtick` events reach the controller while the
   process has no visible window.
4. A worker can deliver target lifecycle and focus events without blocking the
   tray/event loop.
5. A second process forwards CLI commands to the running instance and exits.
6. Configuration reload is atomic and retains the last valid state on failure.
7. Start-at-login does not open a console or settings window.
8. A per-user Windows installer installs, upgrades, and removes the prototype.
9. The tray shell at least compiles and displays on one Linux desktop.

## Proposed Cargo workspace

```text
crates/
├── hudswitch-core/
├── hudswitch-windows/
├── terminalhud-core/
├── terminalhud-windows/
├── terminalhud-tray/
└── terminal-adapter-windows-terminal/
apps/
└── terminalhud/
```

`terminalhud-tray` depends on product application services, not the reverse.

