# Requirements

Status: agreed specification, including [Active creation](decisions/0009-active-hud-creation.md).

## Product scope

- The product name is `TerminalHUD`.
- The implementation language is Rust unless a prototype demonstrates a
  fundamental feasibility blocker.
- TerminalHUD and HudSwitch are cross-platform projects.
- Windows 11 with Windows Terminal is the first supported platform milestone.
- Platform support is implemented through OS/window-system backends and terminal
  adapters rather than platform conditionals in the portable state machine.
- TerminalHUD manages one dedicated Windows Terminal HUD window in the initial
  Windows release.
- The HUD is placed on the primary monitor.
- TerminalHUD is installed per user and starts automatically with Windows.
- Every newly created HUD launches Active and takes focus, including startup
  and relaunch after the managed terminal closes. Initial state is fixed for
  the MVP; existing targets can still become Passive or Hidden.
- TerminalHUD provides a notification-area icon and native tray menu.
- Initial configuration is a documented JSON file with CLI commands to open,
  validate, and reload it. A graphical settings window is not required for the
  first release.
- HudSwitch is a separate, cross-platform generic library in the same monorepo.

## Window states

The managed target has three user-visible states:

| State | Visible | Keyboard input | Mouse input | Terminal background |
| --- | --- | --- | --- | --- |
| Active | Yes | HUD receives it | HUD receives it | 35% opaque by default |
| Passive | Yes | Underlying app receives it | Clicks pass through | Fully transparent |
| Hidden | No | Underlying app receives it | Underlying app receives it | Not applicable |

The active background opacity is configurable and defaults to 35%. The passive
background opacity is zero. Terminal text remains opaque.

The Hidden state remembers whether it was entered from Active or Passive. The
visibility shortcut restores that remembered state.

## Keyboard behavior

Default shortcuts are configurable. The shortcuts below are the Windows
defaults; other platforms use platform-appropriate equivalents while preserving
the same transitions.

### `Win+Backtick`: interaction toggle

- Passive becomes Active.
- Active becomes Passive.
- Hidden becomes Active.
- If no target window exists, TerminalHUD launches it Active.

### `Win+Shift+Backtick`: visibility toggle

- Passive becomes Hidden and remembers Passive.
- Active becomes Hidden and remembers Active.
- Hidden restores the remembered state.
- If no target window exists, TerminalHUD launches it Active.

### Prompt-aware Escape

- Escape at an empty, idle prompt in a supported shell changes Active to
  Passive.
- Escape does not hide the HUD.
- Escape inside Neovim, `less`, `fzf`, SSH, or another interactive terminal
  program is delivered normally.
- `Win+Backtick` remains the unconditional way to leave Active when prompt-aware
  Escape is unavailable.

Initial Windows-milestone shell integrations:

- PowerShell with PSReadLine
- WSL Bash
- Git Bash

Shell integrations are installed explicitly with a preview and a backup. They
must not silently edit shell startup files.

## Focus behavior

- Entering Active focuses Windows Terminal.
- Entering Passive restores focus to the application that was active before the
  HUD was activated, when that window is still available.
- Hiding an Active HUD restores focus to the underlying application.
- Restoring a Hidden HUD that remembered Active focuses the HUD again.
- If the terminal unexpectedly loses focus while Active, TerminalHUD
  automatically changes it to Passive.

## Presentation

- The HUD initially launches fullscreen on the primary monitor with the taskbar
  hidden.
- Windows Terminal chrome is initially hidden.
- Windows Terminal's native fullscreen and Focus Mode commands remain usable.
- The HUD remains topmost while Active or Passive.
- Passive mode is click-through.
- Hiding uses window hide/show behavior rather than closing or minimizing, so
  terminal sessions and scrollback remain intact.

## Windows Terminal ownership

- TerminalHUD launches a dedicated, named Windows Terminal window rather than
  attaching to an arbitrary existing terminal.
- Use the user's installed Windows Terminal and its normal global settings. Do
  not install a duplicate runtime. The HUD owns one window; its owner process
  may also own ordinary Terminal windows.
- Unrelated Windows Terminal windows are never modified or terminated.
- Preserve the selected normal profile's fonts, colors, cursor, padding, images,
  command, directory, environment and other applicable settings, including
  unknown future properties. Only the minimum HUD presentation/input behavior
  may override them.
- TerminalHUD detects Windows Terminal's configured default profile and uses
  the corresponding HUD variant by default.
- TerminalHUD generates HUD variants for every discoverable, launchable Windows
  Terminal profile.
- Normal profiles are not modified.
- TerminalHUD installs profiles and schemes using Windows Terminal JSON fragment
  extensions rather than rewriting the user's main `settings.json`. This existing
  policy cannot represent every profile because user defaults outrank fragments.
  [The proposed owned-entry amendment](decisions/0011-owned-profile-entries.md)
  provides a reviewed alternative, but activation requires explicit setup consent.

## Lifecycle

- Closing the managed terminal leaves TerminalHUD running in the notification
  area.
- The next interaction shortcut relaunches the terminal Active.
- The next visibility shortcut relaunches the terminal Active.
- Quitting TerminalHUD restores the managed window's original styles and leaves
  the terminal running as a normal window.
- A crash or forced exit must not intentionally leave a terminal permanently
  click-through. Recovery behavior must be designed and tested.

## Elevation

- A non-elevated TerminalHUD manages a non-elevated Windows Terminal.
- Managing an elevated Windows Terminal requires TerminalHUD itself to be
  elevated.
- Elevation is explicit; TerminalHUD does not request elevation at every login
  by default.

## Settings and installation

- Settings are stored at `%LOCALAPPDATA%\TerminalHUD\settings.json`.
- The initial Windows build uses a per-user installer. Other platforms use an
  appropriate per-user package or installation model.
- TerminalHUD exposes a same-user local IPC command for shell integrations,
  conceptually `terminalhud passive`.
- Start with Windows, shortcuts, active opacity, default HUD profile, and
  relevant lifecycle behaviors are configurable.

## Initial non-goals

- More than one simultaneous HUD target
- Monitors other than the primary monitor
- Perfect outlined or shadowed glyph rendering
- Modifying ordinary Windows Terminal profiles
- Silently modifying shell configuration
- Publishing a stable HudSwitch package before its API has been exercised by
  TerminalHUD
- Shipping Linux or macOS in the first Windows milestone
- Embedding or implementing a terminal emulator in the MVP

