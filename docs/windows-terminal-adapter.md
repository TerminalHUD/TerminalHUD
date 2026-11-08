# Windows Terminal adapter primitives

The adapter separates read-only `profiles` parsing/preview, pure `launch`
identity/correlation, and Windows-only `native` process/window observations.
It has no runtime dependency on the Win32 backend or app. The intended app
composition converts a verified delegated launch into the backend's receipt;
title, class, and launcher PID alone never authorize acquisition.

`Catalog::parse` consumes supplied JSONC. It merges explicit profile defaults,
but does not resolve generated profiles, installed fragments, or dynamic sources.
A dynamic source without its own command line remains unresolved even when the
global defaults contain a command line. No shell command is evaluated by parsing.
`Catalog::preview` rejects unresolved/elevated sources and GUID collisions. It
creates a separate profile, strips source/update identity, explicitly disables
elevation, and overrides focused/unfocused HUD appearance. Original settings and
profiles remain untouched. It is a preview, not a proof that Terminal loaded it.

`LaunchPlan` requests a fresh named fullscreen window at the supplied primary
monitor origin. `native::start` supplies `SW_SHOWNOACTIVATE` before Passive
creation; the caller must first independently verify the runtime profile exists.
The returned process can delegate to another Terminal process. Native correlation
retains live process identities, checks creation times during ancestry traversal,
excludes preexisting HWNDs, and checks the exact installed executable in addition
to a nonce/profile/session shell handshake. It does not make intercall HWND
lifetime changes atomic. The unfinished example's `probe` command refuses to
launch; `preview` only writes a new directory below the native temporary folder.

## Installed-version source expectations

These observations use Microsoft Terminal v1.24.12741.0, commit
`7ff11ee107c984e236960ec948b450ccbc10187b`. They are static source expectations,
not native GUI/input/rendering evidence:

- [WindowEmperor](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/WindowEmperor.cpp)
  forwards the startup show command during command-line handoff.
- [AppHost](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/AppHost.cpp#L1170-L1207)
  uses that command and skips its foreground request for `SW_SHOWNOACTIVATE`.
  Actual foreground monitoring before/during creation is still required.
- [GetProfileForArgs](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettings.cpp#L735-L780)
  can fall back from an unknown requested profile. With an explicit command line
  it may select a command-matched profile or `ProfileDefaults`; otherwise it
  selects the ordinary default. Supplying a fixture command alone cannot prove
  non-elevated ConPTY launch or absence of inherited launch behavior. Do not
  launch against an unverified fragment cache. No safe runtime-catalog guard or
  alternate-connection/startup-action validation is implemented yet.
- [AppLogic](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalApp/AppLogic.cpp#L304-L340)
  watches the main settings filename. A fragment on disk is insufficient reload
  evidence; do not write main settings or restart unrelated sessions to force it.

Native proof must separately establish fullscreen bounds, transient focus safety,
Active/Passive/Hidden observations, actual click routing, background transparency
with opaque text, session survival, and exact owned-file/session cleanup. Existing
Terminal DirectComposition/layered-window renderer compatibility is unproven.
Stable/Preview and reload coverage must be recorded individually.

## Focused checks

After the workspace's locked dependency fetch, run:

```sh
cargo test -p terminalhud-windows-terminal
cargo clippy -p terminalhud-windows-terminal --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check_repository.py
git diff --check
```

The same Cargo test/lint commands run on native Windows. Six pure tests cover
profile preservation/override, rejected launch sources, dynamic/default command
ambiguity, isolated arguments, launch correlation, and malformed identities.
They do not launch Terminal. The optional `terminal-proof preview <absolute-exe>`
example creates a synthetic fragment preview and record in a fresh task directory;
it never installs a fragment or edits main settings. Its `--shell` fixture checks
nonce, `WT_PROFILE_ID`, and `WT_SESSION`, then waits for a nonce-scoped cleanup
request. It must only be launched after the runtime-catalog guard is implemented.
