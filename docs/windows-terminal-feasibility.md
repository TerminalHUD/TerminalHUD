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

The [show8 attempt](../tests/terminalhud-tests/windows-terminal/evidence/show8.txt)
is **inconclusive**: it logged only an observation timeout. No creation-intent
trace, session marker or exit marker appeared; no own fixture/portable process
remained. Code flow suggests the owned-sink foreground wait timed out before
launch, but that phase was not explicitly logged. This does not establish whether
`SW_SHOWNA(8)` preserves foreground. No forced focus or retry was performed.

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
revisit the inconclusive documented show8 diagnostic with adequate owned foreground
rights; or obtain a product decision to start Absent and create Active on user
request. The last changes current requirements and does not prove Passive launch.
