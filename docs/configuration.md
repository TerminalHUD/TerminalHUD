# Configuration

The initial Windows implementation stores per-user configuration at:

```text
%LOCALAPPDATA%\TerminalHUD\settings.json
```

The schema below illustrates the initial Windows configuration and may change
when implementation begins. It records current product decisions without
committing to property names.

```json
{
  "version": 1,
  "hotkeys": {
    "toggleInteraction": "Win+Backtick",
    "toggleVisibility": "Win+Shift+Backtick"
  },
  "startup": {
    "startWithWindows": true
  },
  "display": {
    "monitor": "primary",
    "initialPresentation": "fullscreen"
  },
  "terminal": {
    "activeBackgroundOpacity": 35,
    "passiveBackgroundOpacity": 0,
    "defaultHudProfile": "auto"
  },
  "lifecycle": {
    "keepRunningWhenTerminalCloses": true,
    "leaveTerminalOpenOnExit": true
  }
}
```

## Configurable values

- Interaction shortcut
- Visibility shortcut
- Start with Windows
- Active background opacity
- Default HUD profile
- Installed shell integrations
- Elevation/restart preference where safe

## Configuration commands

The first release should provide commands resembling:

```text
terminalhud config path
terminalhud config open
terminalhud config validate
terminalhud reload
terminalhud integrations list
terminalhud integrations preview <shell>
terminalhud integrations install <shell>
terminalhud integrations remove <shell>
```

Invalid configuration must not crash the running HUD or silently replace the
file. TerminalHUD retains the last valid in-memory configuration, reports the
validation error through logs and tray status, and waits for a corrected file.

## Fixed initial Windows-milestone values

- One HUD target
- Every newly created HUD starts Active, including shortcut-triggered relaunch
- Primary monitor
- Passive background opacity of zero
- Windows 11 support
- Per-user installation and settings

Initial state is fixed for the MVP, as recorded in
[ADR 0009](decisions/0009-active-hud-creation.md). Passive and Hidden apply to
existing targets; they are not configurable creation states.

The portable configuration model should not expose Windows paths, virtual-key
codes, or `HWND` values. Platform-specific serialization translates portable
shortcut and display concepts into native representations.

## Graphical settings

A graphical settings window is not required for the first release. JSON is the
source of truth. A future UI must edit the same validated configuration model
through TerminalHUD core rather than introduce a second settings system.

## Notification-area menu

Initial menu proposal:

```text
Activate / Make Passive
Show / Hide
Launch / Relaunch
Open Configuration
Reload Configuration
Validate Configuration
Restart as Administrator
Start with Windows        ✓
Quit
```

