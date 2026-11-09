# Per-user Windows package

This is a reviewable development package, not a public release. No command here
needs elevation, changes PATH, installs shell integrations, or kills a process.
Package staging and installation do not edit Terminal settings. The separate
profile setup below would edit them only after explicit setup consent. Native installation and startup application require the final
reviewed operator authorization. Do not change PowerShell execution policy to
run these scripts; use an approved local checkout or signed distribution under
your existing policy.

From native Windows PowerShell, build both release binaries and stage a package:

```powershell
./packaging/windows/build.ps1 -Output C:\staging\TerminalHUD
```

The builder requires committed tracked source, records the source commit and
development version in `BUILD-INFO.json`, gathers Rust dependency license texts,
and refuses existing outputs. It does not bundle or download Windows Terminal;
the installed Windows Terminal is required when using the application. No release
tag or version is published.

Plan installation without writing files or the registry:

```powershell
C:\staging\TerminalHUD\installer\manage.ps1 -Operation install -Package C:\staging\TerminalHUD
```

After review and operator authorization, repeat with `-Apply`. Installation uses
the current user's Local App Data known folder (usually
`%LOCALAPPDATA%\TerminalHUD\install`), never a system location. Existing
`%LOCALAPPDATA%\TerminalHUD\settings.json` is preserved byte for byte. The
manifest's `build_id` selects `install\versions\<build_id>`. Both executables,
licenses and installer scripts are copied there. A per-user Start menu shortcut named `TerminalHUD` launches the exact versioned
GUI without arguments; its path, target and SHA-256 are receipt-recorded.
Unowned or modified shortcuts conflict and are preserved.

Launch TerminalHUD from the Start menu, or the installed `terminalhud-shell.exe` in that version directory with no
arguments, or use its sibling `terminalhud.exe` for commands. The CLI and GUI
must remain together. The installed path requires the separate profile setup
below; development/prototype commands are not install entry points. The first
normal launch applies `start_with_windows`
through the receipt-bound service. Installation itself never registers startup
or launches the app.

## Installed Terminal profile setup (proposed)

The profile-storage decision in [ADR 0011](../../docs/decisions/0011-owned-profile-entries.md)
is proposed and requires explicit user approval before use against real settings.
The local commands below prepare one separate TerminalHUD-owned profile while
leaving ordinary profile bytes unchanged:

```powershell
terminalhud.exe profiles plan
terminalhud.exe profiles apply
```

`plan` reads inputs and reports the intended operation without shell commands or
environment values. `profiles plan --details` additionally prints the exact owned
entry locally and can contain private profile values; do not publish that output.
After review and approval,
`apply` writes a staged replacement with an exact backup and a receipt for the
owned entry. A pending journal or a modified/missing owned entry stops automatic
updates. Keep backups and the receipt for recovery. After setup, configuration
reload can update the unchanged owned entry; a live session is kept open and
changes to its launch/appearance take effect on the next launch. An existing
Terminal host may need a normal settings reload before it recognizes the profile.
TerminalHUD never restarts or kills that shared host. Runtime behavior still needs
validation with the installed Terminal and real session preservation.

## Startup ownership

`StartupRegistration::for_current_install()` validates the current executable,
sibling GUI and installation receipt without writes. `prepare(enabled)` reads
only; `commit(change)` revalidates the receipt and expected old registry value,
then writes only `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value
`TerminalHUD`. Its REG_SZ data is exactly the double-quoted installed GUI path,
with no arguments. Other registry types, commands or arguments conflict and are
preserved. Development/prototype hosts must never call commit.

Prior version commands remain owned through the receipt, allowing a new normal
launch to update the startup path. The service verifies the result after write
and reports write/verification failures rather than claiming success. Windows
has no atomic registry compare-and-swap; an unrelated writer racing between the
last read and write cannot be excluded. No broader registry ownership is claimed.

## Upgrade, uninstall, and cleanup

Install a new package with a different build identity using the same plan/apply
commands. A version directory is never overwritten; all older application files
remain available to live terminals. The next normal GUI launch can update its
owned startup command. Quit the old tray owner before launching the new one;
upgrading does not terminate the tray or terminal sessions automatically.

```powershell
C:\staging\TerminalHUD\installer\manage.ps1 -Operation uninstall
C:\staging\TerminalHUD\installer\manage.ps1 -Operation uninstall -Apply
```

Before uninstall, review and explicitly remove the unchanged owned Terminal
profile with `terminalhud.exe profiles remove-plan` followed by
`terminalhud.exe profiles remove`. Removal retains an exact backup and does not
close live sessions. An edited or missing entry is a conflict and remains
untouched; a pending journal requires recovery before uninstall. The uninstaller
refuses deactivation or cleanup while the profile receipt or pending transaction
remains, so the installed CLI is available to complete that work.

Uninstall removes only the exactly owned startup value and Start menu shortcut,
and marks the installation
inactive. It reports **pending cleanup** and retains all versioned payloads,
settings, and terminal sessions. An unlocked CLI executable can still be
referenced by a live shell, so unlocked files alone are insufficient proof that
removal is safe. An unowned startup conflict aborts uninstall without mutation.
No uninstalled payload can prepare a new startup registration.

After closing all sessions that reference these versions, plan cleanup, then
explicitly confirm they are closed:

```powershell
C:\staging\TerminalHUD\installer\manage.ps1 -Operation cleanup
C:\staging\TerminalHUD\installer\manage.ps1 -Operation cleanup -Apply -SessionsClosed
```

Cleanup removes only receipt-listed files whose SHA-256 still matches. Modified,
unowned, or in-use files remain and are reported as pending. Directories and the
ownership receipt remain; no recursive deletion reaches user files. Keep the
staging package until cleanup completes so the management script itself is not
in use from an installed version. The shortcut is created only during explicit installation.

## Recovery and validation

An exclusive operation lock serializes installer actions. A crashed operation may
leave `install.operation.lock`; verify no installer is running before manually
removing that task-owned lock. The receipt is atomically replaced before copying
payloads, so an interrupted copy remains identifiable. Do not launch an incomplete
version. Plan uninstall and cleanup from the staging package, then reinstall with
a fresh build identity. A partial registry/write failure is reported; inspect a
new read-only plan before retrying. Never remove the receipt to bypass a conflict.
Reparse points, traversal, duplicate paths, Windows device names, altered package
hashes, and unowned install directories are rejected.

`tests/packaging.tests.ps1` uses only a unique temporary directory and a fake
startup value. Hosted Windows CI runs it under native PowerShell. The native builder CI check retains a seven-day review artifact containing
real CLI/GUI binaries and no bundled Terminal runtime;
it is not a distributable product or release. Native startup
unit tests use fake values; no automated test changes the live Run key. A
portable policy harness can also run with:

```bash
rustc --edition 2024 --test packaging/windows/tests/startup_policy.rs -o /tmp/terminalhud-startup-tests
/tmp/terminalhud-startup-tests
```

The package also includes the explicit shell integration manager at
`integrations/shells/manage.ps1`. Installing the application does not change shell
profiles. See the [shell integration guide](../../integrations/shells/README.md)
for preview, backup, install and remove operations.
