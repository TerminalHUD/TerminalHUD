# Controlled Windows Terminal portable proof

A separate, official unpackaged ZIP in a fresh task directory can avoid trusting
an installed process's cached profile catalog. Microsoft's [portable-mode
contract](https://learn.microsoft.com/en-us/windows/terminal/distributions)
keeps its settings beside the executable. The pinned
[FileUtils implementation](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/FileUtils.cpp#L20-L39)
requires an unpackaged executable plus adjacent `.portable` marker. The pinned
[WindowEmperor](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/WindowsTerminal/WindowEmperor.cpp#L309-L358)
includes the unpackaged executable path in remoting identity, separating a fresh
path from the installed distribution's running server.

This is a fixture precondition, not a public runtime-profile recognition API.
Before running it, verify the exact official asset's release metadata, complete
ZIP SHA256, and Microsoft Authenticode signature. Use exclusive task directories,
reject unsafe archive paths/reparse entries, and create only adjacent synthetic
settings. The fixture profile selects its explicit command; both the profile and
profile defaults select `elevate:false` and zero-GUID `connectionType`. Set empty
`startupActions` and `firstWindowPreference:defaultProfile`; pass the fixture
command explicitly on the CLI. Never select an ordinary user shell as fallback.

Portable settings still read external fragments. The pinned
[loader](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L330-L445)
searches shared fragment directories and app extensions. The proof's read-only
`sources` guard enumerates those namespaces and rejects unknown ones or discovery
errors before launch. Disable all observed namespaces and built-in dynamic
sources in the synthetic settings; repeat discovery afterward and reject changed
observations. This does not protect against a hostile concurrent configuration
change and is not a security boundary. No normal settings, fragments, aliases,
registry entries, running sessions, or installed executables may be changed.

The foreground monitor installs an out-of-context foreground WinEvent hook on a
short-lived message-pumping thread before creation. It records intermediate
foreground HWNDs, rather than relying only on the final foreground after launch.
Stop/join it after the bounded launch observation. Only the probe owns this
observer; the production backend retains the caller-owned event-loop contract.

Portable rendering evidence would apply to that exact same-version fixture.
It cannot establish installed Stable fragment-cache recognition, Preview behavior,
or complete ordinary-profile cloning. Those require separate evidence.

Terminal can normalize and rewrite its own synthetic settings, including adding
inbox profiles. Its pinned [loader](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L849-L857)
clears commandline/name/source/GUID from profile defaults. A default command line
is not a launch-safety guarantee; use the explicit CLI fixture command with known
non-elevated ConPTY defaults. Preserve normalized bytes as evidence; reusing the
fixture requires reviewing/restoring only owned configuration and stale markers.

After preparing the reviewed isolated configuration, build the native example:

```powershell
cargo build -p terminalhud-windows-terminal --example terminal-proof --locked
terminal-proof diagnose <absolute-task-directory> 4
# Separately reviewed comparison only:
terminal-proof diagnose <absolute-task-directory> 8
```

The final argument allows only `4` (`SW_SHOWNOACTIVATE`) or `8` (`SW_SHOWNA`),
which are distinct [documented show commands](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow).
This is a diagnostic-only example, separate from the guarded normal launch APIs.
It shows the owned blue primary-monitor sink and dedicated test Terminal, records
numeric/classified foreground events, then requests nonce fixture exit. It has
no backend construction, styles, state transitions, renderer checks or input.
Unknown namespaces, unsafe startup settings and stale protocol markers refuse
execution. Native CI compiles it but never runs this desktop experiment.


## Foreground setup and observation phases

The diagnostic requires an explicit show command; omission never selects a
comparison implicitly. Before creating Terminal it records its selected mode,
validated setup, sink creation, the sink foreground-request result and actual
foreground readiness. It verifies the sink's unique class, owning process/thread
and live thread before one ordinary `SetForegroundWindow` request. If the owned
sink does not become foreground within 3 seconds, it logs `prelaunch-refused`
with the observed HWND and does not create Terminal. No retries, global input,
`AttachThreadInput`, focus-lock changes or focus restoration are used.

The observation hook begins only after that setup. A target foreground event
remains a target-activation failure even if a later event restores the sink.
Any unrelated foreground event or unrelated final foreground invalidates the
observation; when target activation and unrelated activity both occur, both are
reported. Only a sink-only trace with final sink can report preserved foreground.

The sink owns its shutdown: Drop signals a private channel and joins its thread;
that thread verifies exact process/thread/class before destroying its window.
It sends no close message to a stored HWND. Already-closed or mismatched windows
are reported and left untouched. This reduces stale-handle risk; separate Win32
identity queries are not an atomic lifetime guarantee. Namespace and preexisting
Terminal witness checks run after sink teardown even on a prelaunch refusal.

The single fullscreen `8` diagnostic with these phases was subsequently reviewed
and executed. It confirmed target activation; see [the feasibility evidence](windows-terminal-feasibility.md).
Any next control requires its own reviewed arguments, fresh preflight and desktop
slot authorization; the diagnostic is not a product launch bypass.


## Explicit windowed comparison control

The diagnostic-only trailing `--windowed-control` flag is accepted only with
show command `8`, and rejects extra/unknown arguments before setup. It removes
exactly `--fullscreen` from the known launch-plan prefix before appending the
same explicit fixture executable/`--shell`/task/nonce arguments. Omitting the
flag keeps the existing fullscreen diagnostic. Production adapter APIs and
launch-plan construction do not use this comparison option.

```powershell
terminal-proof diagnose <absolute-task-directory> 8 --windowed-control
```

Creation and result logs explicitly label `WindowedControl`. This compares the
fullscreen startup hypothesis only; windowed foreground preservation cannot
establish the required initial fullscreen Passive presentation. Before a separate
reviewed/serialized run, archive the previous task's normalized settings, generated
state, markers and logs byte-for-byte; verify exact observed hashes and no owned
processes; restore only its reviewed synthetic settings and remove only the
explicitly archived task-generated state/nonce markers. Preserve all global
configuration and recheck namespaces, binary hashes and external baselines.


The reviewed single windowed show8 control also activated the exact target;
[the trace](../tests/terminalhud-tests/windows-terminal/evidence/windowed-show8.txt)
contains its explicit mode label, successful sink setup and strict classification.
Both controls remain diagnostic-only failures; production Passive launch stays
unsupported before process creation. The task's original/normalized settings,
state, markers, logs and each fixture executable are archived separately.

## Normal Active acceptance path

The user selected Active for every fresh HUD target. The `run` proof uses the
normal adapter's `start_with_command` Active entrypoint, not a diagnostic show
flag override. It verifies the fresh HWND/executable/retained owner ancestry and
nonce/profile/session before constructing the backend's delegated-launch receipt.
A real HudSwitch controller captures the owned sink foreground before acquisition,
then drives the existing Windows backend. No production policy is implemented in
this probe, and an explicit Passive creation request remains unsupported.

```powershell
terminal-proof run <absolute-task-directory>
```

This command is for a separately reviewed and authorized desktop session. It
checks primary fullscreen bounds, Active foreground/topmost presentation, Passive
focus restoration/click-through, Hidden restoration and same-session survival.
It attempts exactly one physical mouse click only after the hit-test and owner
checks identify its own Passive sink; no keyboard or global shortcut is injected.
A failed focus, identity, layering or renderer check stops the sequence and
attempts controller restoration plus explicit nonce cleanup.

The synthetic renderer check reads only points covered by the verified Terminal
or owned blue sink. It expects black background at 35% opacity over RGB
(40,160,220) to produce approximately (26,104,143) when Active, and the unchanged
blue background when Passive, allowing six levels per color channel. It also
requires at least ten sampled near-white fixture text/block pixels (all channels
at least 245). These are bounded pixel observations, not a general color-management
or complete glyph-rendering guarantee; unexpected composition fails the proof.
No whole-desktop screenshot or unrelated window content is saved.

Quit must restore owned style/visibility changes while leaving the exact session
alive, before the fixture is explicitly asked to exit. The outer runner must
retain a deadline, guard all exact task hashes/markers/processes, and compare
external configuration and existing-Terminal witnesses. Old show-flag/windowed
controls remain only until the separately assigned cleanup removes them from
the final probe; historical raw traces remain evidence, not product launch paths.
