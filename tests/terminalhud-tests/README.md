# TerminalHUD tests

Product orchestration and adapter suites live here. Register real suites with
`[[test]]` targets in their owning product crate. Keep generic HudSwitch behavior
in its own core/backend tests; product suites may compose those components.


`app-shell/commands` exercises the portable product command/controller seam.
`app-shell/ipc` covers native same-user IPC, lifecycle, ACL denial, malformed
clients and reply delivery. `app-shell` also contains the optional native/Tao
loop comparison. See [the shell guide](../../docs/application-shell.md) for the
security boundary and [building](../../docs/building.md) for exact commands.

Ordinary workspace tests run portable suites and unattended native shell tests.
The visible tray/menu test is explicitly ignored: reserve the desktop slot and
run it alone in a real Windows session. It owns and cleans its spawned process,
checks popup ownership before targeted messages, and never drives unrelated UI.
