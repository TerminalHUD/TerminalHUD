# Windows Terminal startup feasibility evidence

Normal Passive fullscreen launch is unsupported and fails before process creation.
This prototype has not met the real-Terminal presentation acceptance criteria.

## Environment and provenance

Tests used Windows 11 build 10.0.26200.0 and native MSVC Rust 1.98.0. Read-only
discovery found installed Stable package 1.24.12741.0 and no Preview installation.
To avoid trusting a running installation's cached profiles, experiments used the
same official release's isolated portable ZIP and synthetic non-elevated ConPTY
profile. Installed settings, ordinary profiles and global fragments were not edited.

[Official asset metadata](https://api.github.com/repos/microsoft/terminal/releases/assets/604365806)
identifies `Microsoft.WindowsTerminal_1.24.12741.0_x64.zip`, 11,656,229 bytes,
SHA256 `7aaa8321064c45d5af79205db13deb87f146f644bd8abdfae29f0155d7ce744e`.
The extracted executable's raw SHA256 was
`4ad317d9dacedca1e68dcc2405d6d364a20599b646b8205feae829cf18ef4958`.
Existing SDK SignTool `verify /pa /v` succeeded with no warnings/errors and a
Microsoft Corporation signing certificate. The PE file version was
1.24.2610.01001, distinct from the release/package version above.

## Observations

The [show4 diagnostic log](../tests/terminalhud-tests/windows-terminal/evidence/show4.txt)
contains the decisive trace:

```text
CREATE_INTENT Passive STARTF_USESHOWWINDOW SW_SHOWNOACTIVATE(4)
FOCUS_EVIDENCE before=7147572 sink=7147572 target=8064844 final=7147572 targetSeen=true events=[8064844]
CLASSIFIED_FOREGROUND_EVENTS ["8064844:own-target"]
```

The target became foreground during creation even though final foreground had
returned to the owned blue sink. The probe did not restore focus. Exact fresh
HWND, owner PID/executable, retained ancestry and nonce/profile/session handshake
were verified before this trace. No backend acquisition, style changes, state
transitions, renderer checks or input followed. The nonce fixture exited and
its dedicated target closed; one preexisting Terminal style/visibility witness
was unchanged. Main settings and two baselined global fragment files were unchanged.

The earlier [show8 attempt](../tests/terminalhud-tests/windows-terminal/evidence/show8.txt)
is **inconclusive**: it logged only an observation timeout. No creation-intent
trace, session marker or exit marker appeared; no own fixture/portable process
remained. Code flow suggests the owned-sink foreground wait timed out before
launch, but that phase was not explicitly logged. This does not establish whether
`SW_SHOWNA(8)` preserves foreground. No forced focus or retry was performed.

A subsequent [phased show8 diagnostic](../tests/terminalhud-tests/windows-terminal/evidence/show8-phased.txt)
reached creation with the owned sink foreground. Its single ordinary foreground
request returned true and the observed HWND matched the sink before measurement:

```text
PHASE sink-foreground-result api_return=true observed=18679874
DIAGNOSTIC_CREATE_INTENT Passive STARTF_USESHOWWINDOW showCommand=8 (ShowNa)
FOCUS_EVIDENCE before=18679874 sink=18679874 target=5048502 final=5048502 targetSeen=true events=[5048502]
FOCUS_CLASSIFICATION TargetActivated { observation_invalid: false }
```

This confirms that `SW_SHOWNA(8)` also fails the Passive foreground requirement
for the pinned fullscreen fixture. Verified Terminal PID 42056 owned HWND
5048502; fixture PID 31108 supplied nonce/profile/session proof. The only
foreground event belonged to that exact target, with no unrelated contamination.
The diagnostic exited 1 without acquiring a backend or applying input/styles.
The fixture exited, target closed, owning-thread sink was destroyed and no task
process remained. Namespace observations, one preexisting Terminal window witness,
main settings and both global fragment byte hashes/inventories were unchanged.

The run used reviewed diagnostic revision
`a685dd0251bd0c16b81adea032d5981dafba788d`, freshly verified portable files and
long-form task path `TerminalHUD-TER9-startup-936bea5c-1738-4f1a-8d49-854b8efb63ff`.
The synthetic settings SHA256 before launch was
`1c2a145684f8d781bfed5124269dff957b80a0a9e8294c554f6eb62988cc10bf`;
fixture executable SHA256 was
`8427fc7f0b1cba40eae91c5a4988e891e30fa674f3cf01c1fb48fdae703263d6`.
Terminal rewrote only its synthetic settings during initialization; original and
normalized bytes, markers and preparation/postrun manifests were retained as
separate task evidence. This is startup-focus evidence, not renderer acceptance.

A separate [windowed show8 control](../tests/terminalhud-tests/windows-terminal/evidence/windowed-show8.txt)
used the same portable path, profile, nonce, original synthetic settings and
explicit fixture command, omitting only the launch-plan `--fullscreen` token.
Its reviewed diagnostic revision was
`5734c688888b203030181498a43f3306a5148461`; fixture SHA256 was
`d68cc3ab3474a924c180d006cb04e57377d1f92c6c220f83554fca208b0d66db`.
The earlier normalized settings/state/markers and fixture bytes were archived,
and original synthetic settings restored before the control.

```text
DIAGNOSTIC_CREATE_INTENT Passive STARTF_USESHOWWINDOW showCommand=8 (ShowNa) presentation=WindowedControl
FOCUS_EVIDENCE before=13439072 sink=13439072 target=15798302 final=15798302 targetSeen=true events=[15798302]
FOCUS_CLASSIFICATION TargetActivated { observation_invalid: false }
```

The control also activated its verified Terminal target: PID 35404/HWND
15798302, fixture PID 47452, session
`{9f237412-0884-476c-99c4-ab9a7bbfe51b}`. Sink foreground setup succeeded, no
unrelated foreground event contaminated the observation, and no backend/style
or input operation followed. Nonce fixture/target/sink cleanup completed with
zero owned processes remaining. Namespace, preexisting Terminal witness and
external configuration hash baselines stayed unchanged. Windowed activation
shows the fullscreen operation is not necessary to reproduce this startup
failure; neither trace identifies the internal cause. This control supplies no
fullscreen presentation or renderer acceptance evidence.

Earlier [identity-guard](../tests/terminalhud-tests/windows-terminal/evidence/initial-identity-guard.txt)
and [foreground-guard](../tests/terminalhud-tests/windows-terminal/evidence/initial-foreground-guard.txt)
logs record refused attempts, not presentation successes. The first rejected
ancestry edge was not localized; its position above/below the owner is unknown.
Focused regressions now require a creation-checked prefix ending at the exact
candidate owner PID/executable and reject wrong/stale/absent anchors.

## Reproduction and limits

Follow [controlled preparation](windows-terminal-proof.md), then explicitly run
`terminal-proof diagnose <absolute-task-directory> 4` or a separately reviewed
`8` comparison. This example owns a separate diagnostic-only startup operation;
normal adapter APIs have no bypass. Both normal entrypoints reject Passive before
spawning, covered by a no-call creation guard and public-entry tests.

The pinned [AppHost source](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/AppHost.cpp#L1193-L1207)
exempts both flags from its late foreground request. Microsoft's
[ShowWindow contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow)
documents their no-activation semantics. These static expectations do not override
the measured show4 activation or identify its internal cause.

Primary fullscreen bounds, Active/Passive/Hidden backend behavior, topmost,
actual click routing, 35%/0% background with opaque text, session survival across
hide/controller quit, and Terminal renderer/layered-style compatibility remain
unproven. Installed Stable fragment reload/cache recognition, Preview and complete
ordinary-profile cloning are also unproven. Unreached renderer/controller/input
probe code was removed; no terminal policy was moved into portable cores.

Follow-up choices require an explicit plan: investigate an upstream startup fix;
investigate a separately reviewed hidden-start diagnostic before any first
Passive presentation; or obtain a product decision to start Absent and create Active on user
request. The last changes current requirements and does not prove Passive launch.


## Source-backed startup lead (not yet measured)

The pinned [TerminalWindow initialization](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalApp/TerminalWindow.cpp#L227-L251)
requests fullscreen during its Initialized event. The page's
[SetFullscreen](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalApp/TerminalPage.cpp#L3985-L3993)
synchronously raises FullscreenChanged, which AppHost forwards to IslandWindow.
Its [_SetFullscreenPosition](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/IslandWindow.cpp#L1091-L1107)
calls `SetWindowPos(HWND_TOP, ..., SWP_FRAMECHANGED)` without `SWP_NOACTIVATE`.
The [Win32 contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
identifies that missing flag as activating behavior. AppHost's later show handler
explicitly yields to the dispatcher before ShowWindow. This fullscreen operation
therefore provides an earlier activation candidate than the late foreground
request already exempted for show commands 4 and 8. The windowed control above also activated, so this fullscreen operation is not
necessary to reproduce the failure. Neither trace proves the internal cause.

The diagnostic now logs the owned sink setup and performs one ordinary,
identity-checked foreground request before starting the Terminal measurement.
Failure to obtain foreground remains a prelaunch refusal, with namespace and
external-window checks still performed. Unrelated transient foreground activity
invalidates observation instead of allowing a false preserved-foreground result.
The earlier timeout remains inconclusive; the subsequent phased show8 result
above provides separate measured activation evidence.

Removing fullscreen and showing a windowed Terminal before verified presentation
would not prove the required initial Passive fullscreen behavior. A hidden,
nonfullscreen startup followed by verified no-activation full-monitor presentation
before its first visibility is an unproven separate hypothesis; the existing
backend does not implement fullscreen geometry. No such control, product change
or requirement relaxation is included in this diagnostic fix.
