# Windows Terminal startup findings

TerminalHUD creates every fresh HUD Active, as accepted in
[ADR 0009](decisions/0009-active-hud-creation.md). Passive and Hidden remain
transitions for existing targets. Normal adapter APIs reject an explicit Passive
creation request before spawning; there is no experimental launch bypass.

## Measured startup behavior

On Windows 11 build 10.0.26200.0, the isolated official Windows Terminal
1.24.12741.0 fixture activated its exact verified target with both
`SW_SHOWNOACTIVATE(4)` and `SW_SHOWNA(8)`. The show8 windowed control also
activated, so fullscreen is not necessary to reproduce the behavior. The exact
internal cause and any fundamental Windows limitation were not established.
An earlier show8 setup timeout was inconclusive and is not activation evidence.

The phased fullscreen and windowed runs verified fresh HWND, executable,
retained owner ancestry and nonce/profile/session before classifying the
foreground trace. Only the owned target activated; no unrelated event
contaminated those observations. No backend, styles or input followed.
Owned fixtures exited and external settings/fragments/window witnesses stayed
unchanged. The original
[fullscreen trace](https://github.com/TerminalHUD/TerminalHUD/blob/dd1ee5d551352c69407ae349323d4ec179f2362e/tests/terminalhud-tests/windows-terminal/evidence/show8-phased.txt)
and [windowed control](https://github.com/TerminalHUD/TerminalHUD/blob/dd1ee5d551352c69407ae349323d4ec179f2362e/tests/terminalhud-tests/windows-terminal/evidence/windowed-show8.txt)
remain in Git history. Their diagnostic commands and event-monitor scaffolding
were removed after the Active-creation decision.

## Coverage boundary

Startup-focus observations do not establish fullscreen bounds, backend
Active/Passive/Hidden behavior, topmost, physical click routing, 35%/0% background
with opaque text, or session survival through hide/controller quit. The
[Active acceptance proof](windows-terminal-proof.md) measures those separately.
Installed Stable fragment-cache recognition, Preview and complete ordinary
profile cloning also require separate evidence. Portable same-version results
must not be presented as coverage of those cases.
