# Configuration

The initial Windows implementation stores per-user configuration at:

```text
%LOCALAPPDATA%\TerminalHUD\settings.json
```

Schema version 1 is implemented by the portable app JSON codec. The complete
example below represents the built-in defaults. This foundation does not yet
read settings files, register shortcuts, apply preferences, or reload the runtime.

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
    "initialPresentation": "fullscreen",
    "targetCount": 1
  },
  "terminal": {
    "activeBackgroundOpacity": 35,
    "passiveBackgroundOpacity": 0,
    "defaultHudProfile": "auto"
  },
  "lifecycle": {
    "initialState": "active",
    "keepRunningWhenTerminalCloses": true,
    "leaveTerminalOpenOnExit": true
  }
}
```

## Schema and validation

`version` is required and must be the integer `1`. All other properties and
sections may be omitted; each missing property takes the example's default.
The document and every section must be JSON objects; positional arrays are
unsupported. `null` is not omission. Unknown properties, duplicate properties, wrong types,
trailing data and unsupported versions are errors, including within sections.
Syntax/type errors include JSON location/property context; semantic errors name
the relevant property. JSON comments are unsupported.

The configurable properties are the two `hotkeys` strings,
`startup.startWithWindows` (boolean), `terminal.activeBackgroundOpacity`
(integer 0 through 100 inclusive) and `terminal.defaultHudProfile` (string).
All other example properties are fixed policy guards: supplying a different
value is an error. Lifecycle preserves sessions by retaining the running
application on terminal close and leaving the terminal open on application exit.
Initial creation is always Active; Passive and Hidden are existing-target states.

Shortcut grammar is case-sensitive: one or more distinct modifiers from `Win`,
`Ctrl`, `Alt`, `Shift`, separated by `+`, followed by exactly one key: `Backtick`,
`A` through `Z`, `0` through `9`, or `F1` through `F24`. Whitespace, aliases,
modifier-only chords and repeated modifiers are rejected. Modifier order does
not matter; serialization orders them Win, Ctrl, Alt, Shift. The two chords must
be distinct after normalization. This supported symbolic subset makes no claim
about native layout mapping, OS availability or registration; it does not change
the currently registered Windows defaults.

Profile selection is the reserved string `auto`, or an opaque ID of 1 through
256 UTF-8 bytes with at least one non-whitespace character and no control
characters. IDs are preserved exactly, without trimming or GUID interpretation.
The core `ProfileId` constructor rejects reserved `auto`. Profile discovery,
identity resolution and launchability checks remain adapter work.

`terminalhud_core::config` owns validated portable values and defaults; it has
no serialization, OS, native-key or filesystem types. The app's `config::decode`
and `encode` provide JSON conversion. Core `Configuration::replace` and app
`replace_from_json` validate a complete candidate before assignment, so failure
preserves the current valid value. These APIs do not write or replace a file.

## Configurable values

- Interaction shortcut
- Visibility shortcut
- Start with Windows
- Active background opacity
- Default HUD profile
- Installed shell integrations (future schema work)
- Elevation/restart preference where safe (future schema work)

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

