# Controlled Windows Terminal acceptance proof

The native `terminal-proof run` example launches a dedicated Active target and
drives the real HudSwitch controller and Windows backend. Run it only in a
serialized interactive desktop session after reviewing the exact fixture and
command. Native CI compiles it but does not run this visible proof.

## Isolated preparation

Use a fresh task directory and an official unpackaged Terminal ZIP. Verify the
release metadata, full ZIP SHA256 and Microsoft Authenticode signature. The
measured release 1.24.12741.0 x64 asset 604365806 is 11,656,229 bytes; its SHA256
is `7aaa8321064c45d5af79205db13deb87f146f644bd8abdfae29f0155d7ce744e`.
The extracted executable SHA256 is
`4ad317d9dacedca1e68dcc2405d6d364a20599b646b8205feae829cf18ef4958`.
Its PE file version 1.24.2610.01001 differs from the release/package version.
Reject unsafe archive paths/reparse entries and record exact prepared-file hashes.

Use full native paths consistently: the current identity check compares
executable paths without resolving short-name aliases. Microsoft's
[portable-mode contract](https://learn.microsoft.com/en-us/windows/terminal/distributions)
requires an unpackaged executable with adjacent `.portable` marker and settings.
The pinned [FileUtils implementation](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/FileUtils.cpp#L20-L39)
and [WindowEmperor remoting identity](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/WindowEmperor.cpp#L309-L358)
explain why a fresh portable path isolates settings and running-server selection.
This is a fixture precondition, not a runtime profile-recognition API.

Prepare only synthetic settings: fresh nonce/profile GUID, explicit fixture
command, `elevate:false`, zero-GUID ConPTY `connectionType`, empty startup actions
and `firstWindowPreference:defaultProfile`. Pass the fixture command explicitly
on the CLI. Terminal [clears invalid profile-default identity/command keys](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L849-L857),
so a default command line alone is not a safe fallback guarantee. Never select
an ordinary shell as fallback or trust a running installation's fragment cache.

Portable settings still [load shared fragments and app extensions](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L330-L445).
Read-only `sources` discovery must succeed and match known disabled namespaces;
unknown namespaces/errors refuse launch. Disable observed and built-in dynamic
sources in synthetic settings. Recheck discovery afterward. This is not a
security boundary against hostile concurrent namespace/configuration changes.
Baseline external settings/fragment bytes and unrelated Terminal witnesses;
never edit normal profiles, main settings, installed executables or sessions.

## Run and evidence

Build with native MSVC Rust using a native Windows target directory:

```powershell
cargo build -p terminalhud-windows-terminal --example terminal-proof --locked
terminal-proof run <absolute-task-directory>
```

The only other operations are `preview <absolute-terminal-exe>`, which creates
a fresh preview without installing it, and the internal nonce shell fixture.
The probe uses normal `start_with_command` Active creation. It verifies fresh
HWND/executable/retained ancestry and nonce/profile/session before constructing
a delegated backend receipt. The controller captures the owned blue sink's
foreground before launch. One ordinary identity-checked sink foreground request
is allowed during setup; failure is a prelaunch refusal, without focus bypasses.

The sequence checks primary fullscreen bounds, Active focus/topmost, Passive
focus restoration/click-through, Hidden memory and same-session survival. Exactly
one physical mouse click is permitted only after hit-testing/PID verification
identifies the owned Passive sink. No keyboard/global shortcuts are injected.
Pixel reads cover only the verified Terminal/owned sink; no desktop screenshot
or unrelated window content is saved. Black background at 35% over RGB
(40,160,220) should approximate (26,104,143); Passive should preserve the blue.
The fixture predicate allows six levels per channel and requires ten near-white
samples (all channels at least 245). These are bounded composition observations,
not complete renderer, color-management or glyph certification.

Any identity, focus, layering or rendering failure stops the sequence and
attempts controller restoration plus nonce cleanup. Quit must restore owned
style/visibility changes while the exact session remains alive, before explicit
fixture exit. The sink shuts down through its owning thread, which validates
PID/thread/unique class; it sends no close message to a saved HWND. Separate
Win32 identity queries cannot make lifetime changes atomic. Namespace/witness
checks run after sink teardown even when setup or proof fails.

The outer runner needs a deadline, exact task-hash/process/marker preflight and
postrun external-baseline comparison. Terminal may normalize synthetic settings
and generate its own state. Before reusing a fixture, archive those exact bytes,
markers, logs and old binary; require the observed hashes and no owned processes,
then restore only the reviewed synthetic settings and remove only explicitly
archived task-generated state/nonce markers. Record each new compiled fixture
hash. Preserve all global configuration throughout.

Portable results cover only that fixture/version. Installed fragment reload,
Preview and complete ordinary-profile cloning remain separate validation work.
See [startup findings](windows-terminal-feasibility.md) for the accepted rationale.
