# Architecture

TerminalHUD is a cross-platform terminal-focused product built on HudSwitch, a
cross-platform generic window-state library. Both projects are cross-platform by
design. Windows 11 and Windows Terminal are the first concrete implementation,
not assumptions embedded in the portable cores.

## Repository shape

The initial monorepo is expected to follow these boundaries. Exact directory
and build-system names depend on the implementation language.

```text
terminalhud/
├── src/
│   ├── hudswitch-core/
│   ├── hudswitch-windows/
│   ├── hudswitch-x11/           # later platform milestone
│   ├── hudswitch-wayland/       # later, capability-limited or cooperative
│   ├── hudswitch-macos/         # later platform milestone
│   ├── terminalhud-core/
│   ├── terminalhud-app/
│   └── terminal-adapters/
│       ├── windows-terminal/
│       ├── wezterm/             # candidate
│       └── kitty/               # candidate
├── tests/
│   ├── hudswitch-core-tests/
│   ├── hudswitch-backend-tests/
│   ├── hudswitch-test-window/
│   └── terminalhud-tests/
├── integrations/
│   └── shells/
├── packaging/
│   ├── windows/
│   ├── linux/
│   └── macos/
└── docs/
```

Only the Windows backend, Windows Terminal adapter, and Windows packaging are
required for the first milestone. Empty placeholder projects for future
platforms are unnecessary.

## Dependency direction

```text
TerminalHUD app --------------------------> TerminalHUD core
      |                                          |
      |                                          v
      +--> terminal adapter -------------> HudSwitch core
      |                                          ^
      +--> platform composition ----------------|
                                                 |
HudSwitch platform backend ---------------------+
```

Rules:

- HudSwitch core depends on no terminal adapter or OS backend.
- HudSwitch backends implement contracts owned by HudSwitch core.
- Terminal adapters implement contracts owned by TerminalHUD core.
- TerminalHUD selects a backend and adapter at composition time.
- No Windows types such as `HWND` cross the portable public boundary. A backend
  may expose opaque handles internally.
- Portable tests use a fake backend; backend integration tests use a generic
  purpose-built test window rather than a terminal.

## HudSwitch core responsibilities

- Active, Passive, and Hidden state machine
- Target and window abstractions
- Capability negotiation
- Transition serialization and rollback policy
- Platform-neutral lifecycle events and errors
- Focus-restoration intent
- Generic launch and window-acquisition contracts

HudSwitch core must not contain:

- Win32, X11, Wayland, or Cocoa types
- Terminal application identifiers
- Shell-specific code
- Terminal opacity, color-scheme, or rendering policy
- Tray UI, product startup policy, or installer behavior

## HudSwitch backend responsibilities

Each backend implements the capabilities available on its platform or window
system:

- global hotkey registration;
- target launch and acquisition;
- topmost behavior;
- pointer click-through;
- show/hide behavior;
- focus capture and restoration;
- lifecycle monitoring;
- original-state preservation and restoration; and
- privilege/permission compatibility reporting.

Backends report capabilities explicitly. They must not pretend an operation is
portable when the platform forbids it. TerminalHUD controls only the dedicated
terminal target it launched; it never selects an unrelated user window.
Nevertheless, a terminal launched as a separate process is still a separate
Wayland client, and the launch relationship does not grant TerminalHUD authority
over that client's native window. A Wayland backend may therefore require
cooperation from a terminal adapter, a layer-shell surface actually hosted by
TerminalHUD, or compositor-specific support.

## TerminalHUD core responsibilities

- Product-level orchestration
- Terminal-adapter contracts
- Portable configuration model
- One-target initial product policy
- Shell-integration orchestration
- Coordination between a terminal adapter and HudSwitch

## Terminal adapter responsibilities

A terminal adapter handles terminal-specific behavior such as:

- discovering, launching, and identifying the terminal;
- creating HUD-specific profiles or configuration;
- translating product appearance settings;
- selecting native fullscreen or presentation features;
- reporting whether the terminal cooperates with click-through and focus APIs;
- detecting when settings require reload or relaunch; and
- offering terminal-specific profile and shell discovery.

The first adapter targets Windows Terminal. Candidate later adapters include
WezTerm and Kitty, but they are not selected merely because they run on multiple
platforms; their control APIs and window-system behavior must be validated.

## Terminal hosting modes

The architecture must not assume that every terminal is always an unrelated
window found after launch. A terminal integration may use one of three modes:

1. **Managed external terminal:** TerminalHUD launches and exclusively manages a
   dedicated terminal process, then the HudSwitch backend controls its native
   window where the platform permits. Windows Terminal is the first example.
2. **Cooperative terminal:** the dedicated terminal remains a separate process,
   but an IPC/plugin/control API asks the window-owning process to apply HUD
   behavior itself. This can satisfy platforms that restrict cross-process
   window control.
3. **Embedded terminal host:** TerminalHUD owns the native window and embeds a
   terminal engine in-process. This provides the strongest cross-platform
   control but expands the product into terminal emulation and rendering.

HudSwitch and TerminalHUD run in the same process in the normal application.
The Windows MVP uses mode 1 with Windows Terminal as a separate managed process.
An embedded terminal is a possible future feature, not a requirement. The MVP
must avoid making modes 2 or 3 impossible, but does not implement them.

## TerminalHUD application responsibilities

- Native tray/status-area icon and menu
- Platform-appropriate startup registration
- JSON configuration, CLI commands, and per-user IPC endpoint
- Platform packaging and elevation/permission UX
- Composition of the selected HudSwitch backend and terminal adapter

## IPC boundary

Shell integrations invoke a lightweight command such as:

```text
terminalhud passive
```

The client connects to the already-running TerminalHUD instance over a
same-user, platform-appropriate local IPC transport. Windows may use a named
pipe; Unix-like platforms may use a Unix-domain socket. Shell integrations do
not manipulate windows directly.

## Packaging HudSwitch

HudSwitch begins as an independent library package inside the monorepo. Its core
and backends have explicit public boundaries and may be versioned separately.
Publishing language-ecosystem packages is deferred until TerminalHUD has
exercised the contracts and exposed incorrect abstractions.

