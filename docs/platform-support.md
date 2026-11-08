# Platform support

TerminalHUD and HudSwitch are cross-platform projects. Cross-platform does not
mean that every desktop exposes the same window-control API. The portable core
defines behavior and negotiates capabilities; platform backends and terminal
adapters implement what is possible.

## First milestone: Windows 11

- Window system: Win32/DWM
- HudSwitch backend: Win32 window styles, focus, hotkeys, visibility, and Z-order
- Terminal adapter: Windows Terminal
- Pseudoterminal ownership: Windows Terminal/ConPTY, not TerminalHUD
- Installation: per-user Windows installer

This is the fully specified initial product milestone.

## Linux/X11

An X11 backend can generally manipulate external top-level windows using X11
and EWMH facilities. Input-shape support can provide click-through behavior.
Terminal behavior and compositor compatibility still require testing.

Potential adapters include Kitty, WezTerm, and other terminals with stable
configuration or remote-control interfaces.

## Linux/Wayland

TerminalHUD never controls an arbitrary existing terminal: it manages the
dedicated terminal process it launched. Wayland nevertheless treats that
separate terminal process as a separate client. Launching it does not grant
TerminalHUD permission to make its window topmost, click-through, hidden, or
focused.

A Wayland implementation may require one of these models:

1. A cooperating terminal exposes the required controls.
2. TerminalHUD owns a layer-shell window and hosts or renders the terminal.
3. A compositor-specific plugin or protocol supplies the missing capabilities.
4. The backend reports unsupported capabilities and offers reduced behavior.

The project must not weaken the core abstraction by disguising this limitation.

Running an embedded terminal engine inside TerminalHUD is the cleanest way to
own the Wayland surface, but it also makes TerminalHUD responsible for terminal
emulation, PTYs, rendering, fonts, Unicode, IME, selection, clipboard,
accessibility, and related behavior. A cooperative separate process can preserve
the window-system ownership benefits without requiring the same OS process.

## macOS

A macOS backend would use native window and global-hotkey APIs. A dedicated
terminal launched by TerminalHUD is still a separate process, so controlling
its native window may require Accessibility permission and may not offer every
behavior available to an application-owned `NSWindow`.
Terminal adapters and permission UX require a dedicated design spike.

## Capability model

A backend should report capabilities resembling:

```text
launch_target
acquire_external_window
set_topmost
set_click_through
show_hide
activate
restore_focus
register_global_hotkey
observe_lifecycle
preserve_restore_native_state
```

TerminalHUD can combine backend capabilities with terminal-adapter cooperation.
Unsupported required capabilities produce an explicit setup error or reduced
mode; they must not fail silently.

## Implementation language

Rust is the selected implementation language. It should be reconsidered only if
the Windows feasibility prototype demonstrates a fundamental blocker that cannot
reasonably be addressed with a contained native boundary.

Reasons it fits the project include:

- a strong type system for state transitions and capability contracts;
- memory safety around long-running tray and window-event code;
- explicit, contained `unsafe` boundaries around native APIs;
- practical cross-platform systems programming; and
- a natural Cargo-workspace monorepo with independently publishable HudSwitch
  crates.

The first release does not require a general GUI toolkit. A focused native tray
crate plus JSON configuration is preferable to Slint, Tauri, or a webview. UI
toolkits can be reconsidered if a graphical configuration editor becomes useful.

Rust does not by itself solve Wayland or cross-process permission limitations,
and its event-loop/packaging choices still need validation. The final language
remains Rust while small spikes prove the Windows backend, tray shell, and
global hotkeys. A non-Windows backend is not required for the MVP.

