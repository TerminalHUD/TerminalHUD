# Thread-owned Windows hotkeys

`hudswitch-windows` registers explicit platform bindings on the caller's thread
queue. It owns no message pump, controller loop, or product shortcut policy.
The selected TerminalHUD defaults and existing user/Terminal settings are unchanged.

## Registration and delivery

Create `HotkeyBinding { id, modifiers, virtual_key }` values and call
`Hotkeys::register(&bindings)` on the authoritative UI/controller thread.
The API always includes `MOD_NOREPEAT`. It validates the whole batch before
mutation: application IDs in `0..=0xBFFF`, recognized modifier bits, virtual-key
range, reserved F12, duplicate IDs, and duplicate chords ignoring `MOD_NOREPEAT`.
`MOD_WIN` is accepted; actual registration may fail with an OS reservation/conflict.

The native call uses a null window handle, so genuine key presses deliver
`WM_HOTKEY` to this thread's queue. The caller pumps messages and maps the owned
ID in `wParam` to its own command. `bindings()` exposes currently owned bindings.
Posted messages are not evidence of physical input or foreground permission.
No callbacks, hidden windows, input injection, or foreground grants are installed.

Microsoft documents [RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)
and [WM_HOTKEY delivery](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-hotkey).
F12 is reserved for debuggers; Windows-key combinations are broadly OS-reserved.
A failed selected binding is reported, never silently replaced. Some system
defaults have special override behavior; callers must choose appropriate chords.

## Ownership and cleanup

`Hotkeys` is neither `Send` nor `Sync`; create, process messages, and clean up on
the same thread. Explicit `unregister()` reports each failed binding/operation/
Win32 code and retains failed entries for retry. Successful cleanup is idempotent.
Drop attempts cleanup, but cannot report errors: explicitly clean up before the
message pump exits when confirmation matters.

Live IDs are reserved across this library's guards on each thread, including
after a failed unregister. Raw Win32 callers must coordinate the same ID
namespace: Windows provides no query that proves an ID was unused, and repeated
registration of an identical thread/ID can accumulate registrations. Never
register or unregister an owned ID through another API while its guard is alive.

On partial registration failure, only this batch's successful prefix is unwound.
`HotkeyRegistrationError` reports the initiating failure and rollback failures;
its `remaining` guard retains incomplete cleanup for inspection/retry. Dropping
that error is best effort too. Failed ID reservations are not falsely released
for reuse; thread exit eventually ends their native lifetime. No unrelated
registration is deliberately unregistered.

## Evidence and limits

Native `cargo test -p hudswitch-windows --test hotkeys --locked` exercises invalid
and duplicate data, same-thread ID ownership, native conflicts, partial unwind,
explicit/drop cleanup and retry after deliberate invalidation of a test-owned
registration. Tests serialize temporary Ctrl+Alt+Shift+F23/F24 registrations,
never press keys, and report an unavailable test chord as a failure. Compile-fail
documentation tests verify thread confinement.

These tests do not exercise selected product keys, physical `WM_HOTKEY` delivery,
foreground eligibility, keyboard layouts, or actual HUD transitions. Those need
a separately coordinated real-user-input application proof. A successful owned
test chord does not establish availability of Win+Backtick or Win+Shift+Backtick.
