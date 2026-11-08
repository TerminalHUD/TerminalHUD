# Win32 backend and generic-window validation

`hudswitch-windows` implements the accepted [portable backend contract](hudswitch-core.md).
It contains no terminal, appearance, tray, IPC, or product configuration policy.
The native API dependency is `windows-sys` 0.61.2, scoped to this Windows crate.

## Ownership and launch boundary

Construct `WindowsBackend<L>` on the caller's authoritative controller/UI thread.
It is thread-confined, starts no permanent event loop, and does not install a
competing input/message pump. The application owns event dispatch: platform
notifications must be queued and delivered with the current core `TargetId`.
Automatic WinEvent monitoring and global hotkeys are not implemented here.

The injected `Launcher` receives Active or Passive creation intent and returns a
`LaunchedWindow`. Passive launch must be nonactivating from creation onward;
the backend rejects a Passive receipt that already owns foreground. The launcher
owns partial acquisition cleanup and must leave any created target ordinary and
usable when it reports an error.

`LaunchedWindow::from_child` verifies that a top-level HWND belongs to the exact
child process the launcher started, duplicating its process handle. For adapters
whose dedicated launch delegates to another process/server,
`unsafe from_verified_launch` is an explicit trust boundary: the adapter must
prove this particular isolated target was produced by its launch, and supply a
handle to the actual owning process. It is not permission to attach an arbitrary
window of the same application. The backend still verifies process/thread
identity and lifetime before applying its own ownership marker.

Acquisition snapshots original owned state before style mutations and registers
an atom-backed, per-acquisition window marker. An existing marker is rejected,
including another controller/process's marker. Every target operation checks the
retained process lifetime, owning PID/thread, and token; HWND existence alone is
insufficient. Destroyed/replaced or unmarked windows are rejected. The registered
atom has an explicit lifetime rather than creating a new global atom for each
window. Dropping tokens releases handles/metadata, not the target process.

Original-state restoration reapplies only style bits the backend attempted to
change, plus topmost/visibility when it changed those. Current unrelated style
bits are preserved. The ownership marker is removed only after successful
restoration. Failure keeps management/snapshot ownership in the core for recovery
or retry. Neither quit nor token destruction closes the target.

## Presentation, focus, and renderer limits

Supported standard top-level windows use topmost, show/hide, and extended-style
operations. Classes using `CS_OWNDC`/`CS_CLASSDC` are rejected before mutation
because they do not support this layered-window route. API and privilege failures
are returned with operation/error information rather than reported as success.

Passive input routing uses `WS_EX_LAYERED | WS_EX_TRANSPARENT`, following
[Microsoft's layered-window semantics](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features).
For an originally nonlayered window, the backend initializes only the layering it
adds at alpha 255 to preserve ordinary opaque appearance. Initialization remains
pending across a failed call so fallback can retry or remove the added style.
Existing layered targets keep their alpha, color-key, and per-pixel attributes;
no `SetLayeredWindowAttributes` call is made for them.

This is not a blanket renderer-compatibility guarantee. Microsoft documents that
[SetLayeredWindowAttributes can disrupt UpdateLayeredWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setlayeredwindowattributes)
until the layer bit is reset. Actual terminal/composition renderer compatibility
must be validated by the adapter. Product opacity controls remain outside this
backend. The generic suite checks existing alpha/color-key preservation;
per-pixel renderer compatibility remains untested.

The backend requests foreground once and confirms the actual result within a
bounded wait. It rejects activation of a disabled/hidden target. Windows can
refuse otherwise eligible foreground changes; refusal is returned to the core
for safe Passive/Hidden fallback. No input-queue attachment, forced retries,
foreground-lock changes, or synthetic user action is used by the backend.
See [the foreground restrictions](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow).

Focus restoration validates the saved process/thread and current window
eligibility, attempts it once, and confirms the result. Focus capture itself is
read-only and adds no marker to an unrelated foreground window. When no saved
foreground can be restored, release hides the target and waits for Windows to
choose another foreground, re-showing it nonactivating if Passive needs it visible.
An already-selected other foreground is left untouched. Asynchronous visibility
and foreground changes are observed before a successful state is reported.
Timeouts return failure; they do not justify a false safe/unmanaged state.

## Automated and explicit desktop checks

Ordinary `cargo test --workspace --locked` runs noninteractive Windows checks and
the portable model suite. Desktop tests are explicitly ignored, so unattended CI
does not silently certify focus or mouse routing.

On the actual Windows desktop, use the native build path from
[building.md](building.md), with a native Windows output directory, then run:

```powershell
cargo build -p hudswitch-windows --example test-window --locked
cargo test -p hudswitch-windows --test windows_backend --locked -- --ignored --nocapture --test-threads=1
```

The fixture launches only its own target, focus-sink, and unchanged-witness
processes. They pump their own Win32 loops and announce readiness. Tests use
process-local DPI awareness, verify the input point resolves to an owned fixture,
and send guarded mouse input only there. The old cursor position is restored
without clicking it. The fixture records delivered `WM_LBUTTONDOWN` messages,
which distinguishes real click routing from style flags or hit-testing alone.
Cleanup requests close and, if needed, terminates only recorded fixture children.

The suite asserts initial Active/Passive/Hidden, saved-focus restoration, actual
Passive click-through to the owned sink, 20 repeated Passive/Hidden transitions,
owned-style restoration while preserving an unrelated style change, an unchanged
witness, safe refusal for an ineligible disabled target, and original layered
alpha/color-key preservation. It also checks competing ownership, detached/live
unmarked targets, and closed processes. Successful repeated Active transitions
are conditional on Windows foreground permission: if denied, the test explicitly
asserts the reported refusal and actual safe Passive fallback and prints that
limit. This is not evidence that a future global-hotkey path will be permitted;
that requires its own application integration test.

Record the actual output, OS/build, compiler, and limits in the tracker handoff.
The generic desktop run does not prove elevated/non-elevated cross-boundary
control, per-pixel renderer compatibility, actual HWND-reuse scheduling, terminal
behavior, automatic lifecycle notifications, or crash/watchdog recovery. UIPI
failures remain explicit; no elevation is requested automatically. A process crash
can leave owned styles on an external target until recovery is designed and
validated; normal explicit quit restoration is the path covered here.
