# Windows Terminal integration

TerminalHUD owns one dedicated Windows Terminal window. It does not attach to
or modify an unrelated terminal.

## Launch model

The conceptual launch resembles:

```powershell
wt.exe -w TerminalHUD -F -p "HUD — <default profile>"
```

Exact arguments must be verified during implementation. The important behavior
is:

- use a dedicated named Terminal window;
- launch fullscreen on the primary monitor;
- select a generated HUD profile variant;
- acquire the resulting top-level window deterministically;
- never rely solely on a process handle returned by the `wt.exe` launcher.

## Appearance

Each generated HUD profile has a configurable Active background opacity and a
fully transparent unfocused appearance. The initial default is:

```json
{
  "opacity": 35,
  "useAcrylic": false,
  "unfocusedAppearance": {
    "opacity": 0,
    "useAcrylic": false
  }
}
```

This lets Windows Terminal perform the live rendering change:

- Active and focused: background is 35% opaque by default.
- Passive and unfocused: background is fully transparent.
- Terminal text remains opaque.

HudSwitch does not implement opacity. TerminalHUD ensures an unexpected focus
loss also changes the logical state to Passive, preventing an invisible
fullscreen input blocker.

Whether fragment edits hot-reload reliably in an already running Terminal must
be tested. State-driven switching between already configured focused and
unfocused appearances must not require a restart.

## Profile discovery

TerminalHUD locates the applicable Stable, Preview, or unpackaged Terminal
settings and reads `defaultProfile`. It resolves that profile GUID against:

- profiles explicitly defined in user settings;
- dynamic Windows Terminal profiles;
- installed JSON fragments;
- detected WSL distributions and other supported profile sources.

If the profile cannot be reproduced safely, setup asks the user to choose a
profile instead of guessing.

## HUD profile variants

TerminalHUD generates a HUD variant for every discoverable, launchable profile.
Normal profiles remain untouched. A HUD variant preserves relevant launch
behavior while overriding HUD-specific presentation.

Examples:

- `HUD — PowerShell`
- `HUD — Ubuntu`
- `HUD — Git Bash`

Only recognized shells receive prompt-aware Escape integration. Other variants
remain usable through the global interaction hotkey.

Generated fragments live under:

```text
%LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\TerminalHUD\
```

The adapter must not rewrite the user's primary `settings.json` merely to add
profiles or schemes.

## Fullscreen and Focus Mode

TerminalHUD initially launches the window fullscreen so the taskbar is hidden.
Windows Terminal's own commands remain available:

- Fullscreen controls taskbar coverage and fullscreen presentation.
- Focus Mode controls Terminal chrome such as tabs and the title bar.

TerminalHUD does not intercept or replace these commands.

## Target acquisition

The adapter supplies HudSwitch with a deterministic matcher based on a unique
window identity, Terminal window class, and launch correlation. A class-name
scan alone is insufficient when the user has other Terminal windows.

If the target closes, the adapter reports it as absent. TerminalHUD stays in the
notification area and relaunches on the next relevant shortcut.

## Adapter implementation boundary

The current [adapter primitives and validation contract](windows-terminal-adapter.md)
separate safe read-only previews from native launch proof. A generated fragment
file does not establish that a running Terminal loaded its profile. An unknown
profile GUID may silently fall back to ordinary defaults, including inherited
launch behavior; native launch must remain blocked until that ambiguity is
resolved without modifying the user's main settings.
