# Application shell prototype

The Windows host provides a native tray and authenticated command transport.
Its explicit prototype mode simulates HUD state without creating a terminal.
The [real development slice](windows-development-slice.md) composes the accepted
controller, backend and adapter using an isolated portable development runtime.
The long-term crate layout remains the one in [architecture.md](architecture.md).

The shared product controller selects Active for any command that creates a
fresh target, including an absent-target visibility toggle. Existing targets
retain Passive/Hidden behavior; an absent `passive` request does not create one.
This prototype demonstrates command policy only, not real window behavior.

## Running

Build both binaries as described in [building.md](building.md). From native
PowerShell, with `CARGO_TARGET_DIR` set to the native build directory:

```powershell
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype run
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype status
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype visibility-toggle
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype interaction-toggle
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype passive
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype quit
```

The console binary prints one reply and exits 0, or reports errors on stderr and
exits 2. `run` starts the sibling GUI binary and waits for an authenticated status
reply from its initialized UI/controller. The GUI binary creates a tray icon and
native menu, with no console or visible main window. Repeated starts contact the
primary instead of adding another icon. Ship both executables together.

Every simulated invocation requires `--prototype` and replies say `mode=prototype`.
Real commands use the separate reserved real scope and say `mode=real`.
Managed state with no confirmed presentation, absent target and unmanaged state
remain distinct in the protocol; a simulated state never claims a real terminal.
The optional `--instance NAME` selects an isolated prototype scope, using 1–32
ASCII letters, digits, underscores or hyphens; it is not an authorization secret.

## Ownership and lifecycle

Product core owns the portable `Command`/`ShellController` seam. App code uses
portable backend contracts through its product-core integration surface. One
native UI thread owns the controller. Menu callbacks and the IPC worker enqueue
bounded requests; WndProc drains them even inside a native menu's nested loop.
A shared reentrancy guard precedes every controller borrow. No producer runs a
second backend/controller loop.

The server retains its first-instance pipe handle across every disconnect and
until the tray/window have been removed. It does not release/recreate the name
between clients. A client acknowledges receipt of its reply before the server
disconnects, because disconnect discards unread named-pipe bytes. A confirmed
quit queues shutdown after that bounded acknowledgement attempt, with a UI timer
fallback. Shutdown disables producers, cancels and joins the server, then drops
tray/window resources and the retained pipe. A failed portable quit does not
authorize exit.

## Local transport and authorization

The endpoint is a local message-mode named pipe scoped by user SID, Windows
session ID and prototype instance. Its protected DACL grants access only to that
user SID; both ends also validate the peer process token's user SID and session.
Remote clients are rejected. The client requests identification-level security
quality of service. A predictable namespace alone does not establish identity.
The protected descriptor and peer checks apply in prototype mode too.

This boundary allows processes running as the same account in the same session;
it does not isolate mutually distrustful programs under that account. An allowed
peer can issue any supported command or occupy the single connection until the
bounded timeout. The owner token check is not an integrity-level/sandbox policy.
No credential prompting, administrator requirement or system-wide ACL changes
are involved.

Frames are fixed-size: requests are 8 bytes (`THP1`, command, zero reserved bytes),
replies 32 bytes (`THR2`, result, explicit runtime mode, status/counters and native hotkey error),
then a 4-byte `THA1` receipt acknowledgement. Unknown commands, wrong versions,
reserved-bit changes, truncated and oversized messages disconnect without a
controller request. I/O uses cancellable overlapped operations with 1500 ms
per-operation deadlines; UI replies have a 2 s deadline; the queue holds at most
64 messages. Idle accept is cancellable. OVERLAPPED storage and buffers remain
alive until cancellation completes. Prototype startup readiness has a 20 s outer bound
plus a currently in-flight bounded transport operation.

A timeout after a valid request is ambiguous: the UI might already have applied
it. Inspect `status`; do not blindly retry a toggle. There is no persistent
command deduplication, network protocol or public compatibility guarantee for
this versioned prototype transport.

Primary references: [named-pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights),
[CreateNamedPipe](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createnamedpipew),
[CancelIoEx](https://learn.microsoft.com/en-us/windows/win32/api/ioapiset/nf-ioapiset-cancelioex),
and [DisconnectNamedPipe](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-disconnectnamedpipe).

## Validation boundaries

Portable tests cover command routing, read-only status, failed quit, strict
request/reply schemas and honest unconfirmed status. Unattended native tests
cover CLI stdout/exit, same-user round trips, duplicate host startup, malformed
and stalled clients, denied anonymous access, delayed reply reading and orderly
quit. These run in the normal workspace checks without visible icons.

The explicitly ignored desktop test creates only its own bounded prototype
host. Windows 11 evidence confirmed no console/main window, IPC while a real
native popup was modal, exactly one native menu action and reply-before-quit
cleanup. UI Automation exposed no descendants for the verified popup on this
desktop. Selection therefore used normal Down/Return keydown/up messages sent
only to that popup after checking its PID, class and lifetime before each
message. It used no global input or private callback invocation. The harness
allows 250 ms for shell icon placement; it does not automate Explorer or other
applications. `--menu-probe` is an explicit prototype-only test hook that opens
the real native menu; it does not select or dispatch commands itself.

The separate [toolkit experiment](ui-toolkit-evaluation.md) established native
and Tao tray-loop compatibility. Real Terminal windows, hotkeys, click-through and focus behavior are composed by
the [development slice](windows-development-slice.md) and require its separate
desktop integration evidence.
