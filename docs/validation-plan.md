# MVP validation plan

These are engineering questions to answer with prototypes and tests. They are
not unresolved product requirements.

## Rust application shell

- Prove a tray-only Rust process on Windows with no console window.
- Compare `tray-icon` with a small Tao event loop against integration with the
  Win32 backend's native message loop.
- Prove single-instance IPC and CLI forwarding.
- Select the per-user Windows installer technology.
- Select and verify start-at-login registration.

## HudSwitch Win32 backend

- Register `Win+Backtick` and `Win+Shift+Backtick` reliably.
- Verify Active, Passive, and Hidden transitions against a generic test window.
- Verify topmost and click-through behavior without corrupting original styles.
- Verify focus capture, restoration, refusal, timeout, and rollback behavior.
- Verify cleanup and recovery after ordinary exit and process failure.
- Test non-elevated and elevated compatibility boundaries.

## Windows Terminal adapter

- Acquire only the dedicated Terminal window across Stable and Preview releases.
- Verify click-through with fullscreen, topmost, and the Terminal renderer.
- Verify live focused/unfocused opacity changes.
- Determine whether fragment opacity edits hot-reload or require relaunch.
- Define and test cloning rules for explicit, dynamic, WSL, SSH, and third-party
  profiles.
- Fall back safely if the configured default profile disappears.
- Verify the primary-monitor fullscreen launch path.

## Shell integrations

- Implement empty-buffer detection in PSReadLine Emacs and Vi modes.
- Preserve Bash Alt/Meta escape-prefix behavior.
- Test completion menus, multiline commands, nested shells, SSH, terminal
  multiplexers, WSL Bash, and Git Bash.
- Make preview, installation, backup, removal, and repeated installation safe.
- Define behavior when the user edits an installed integration block.

## Hotkeys and keyboard layouts

- Verify the physical/default backtick binding across relevant keyboard layouts.
- Define portable JSON spelling for layout-dependent keys.
- Report actionable errors when another application owns a configured shortcut.

## Configuration and diagnostics

- Validate configuration atomically and retain the last known-good state.
- Define diagnostic and rolling-log locations.
- Ensure tray and CLI errors identify the failing property or subsystem.
- Verify that quitting leaves the terminal usable and does not lose running
  terminal work.

## MVP exit criteria

The MVP is technically viable when:

1. The tray process starts at login in Passive without stealing focus.
2. Both global shortcuts implement the documented state machine.
3. Prompt-aware Escape works in PowerShell, WSL Bash, and Git Bash without
   stealing Escape from Neovim.
4. Windows Terminal remains stable through repeated style, focus, opacity,
   fullscreen, visibility, close, relaunch, and exit transitions.
5. Installation and removal leave user Terminal and shell configuration intact.

