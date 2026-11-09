# Real Windows development slice

The application composes the real product controller, `WindowsBackend`, Windows
Terminal adapter, native tray, intended hotkeys and authenticated IPC. This is a
development milestone. Installed Terminal profile recognition, configuration,
installation, startup registration and shell Escape remain later work.

The installed adapter cannot yet prove that a requested fragment is loaded. The
explicit portable runtime below is an isolated development/test precondition;
it is not a new requirement for the eventual installed product. No command
writes ordinary Terminal settings, fragments or shell startup files.

## Prepare an isolated runtime

Use native Windows and a fresh task-owned copy of the verified official
unpackaged Terminal package. The accepted version is 1.24.12741.0; follow the
[proof preparation](windows-terminal-proof.md) for package hashes, signature,
archive safety and read-only external namespace discovery. Do not repurpose an
ordinary portable Terminal directory or a running user's session. Use long native
paths throughout; short-name aliases do not satisfy exact executable matching.

Build both application binaries into native Windows TEMP as in
[building](building.md). Create an empty `.portable` marker beside the dedicated
`WindowsTerminal.exe`. Its `settings/settings.json` must contain one explicit
synthetic profile and the following startup fields:

```json
{
  "defaultProfile": "{FRESH-PROFILE-GUID}",
  "startupActions": null,
  "firstWindowPreference": "defaultProfile",
  "disabledProfileSources": [
    "Windows.Terminal.PowershellCore", "Windows.Terminal.Wsl",
    "Windows.Terminal.Azure", "Windows.Terminal.VisualStudio",
    "Windows.Terminal.SSH", "Microsoft.WSL"
  ],
  "profiles": { "defaults": {}, "list": [] }
}
```

Replace the placeholder with a fresh real GUID. Put these same safe appearance/connection fields in `profiles.defaults` and
its sole `profiles.list` entry, and give the entry that GUID and a descriptive
name. Put `commandline` only in the explicit entry; Terminal strips that invalid
defaults property when normalizing settings. The shared fields are:

```json
{
  "connectionType": "{00000000-0000-0000-0000-000000000000}",
  "elevate": false,
  "opacity": 35,
  "useAcrylic": false,
  "suppressApplicationTitle": true,
  "foreground": "#FFFFFF",
  "background": "#000000",
  "unfocusedAppearance": { "opacity": 0, "useAcrylic": false }
}
```

The explicit profile also needs `commandline` equal to
`"C:\absolute-native-build\debug\terminalhud.exe" --session-bridge` (with those
path quotes encoded in JSON). The bridge path must be the console binary beside the shell binary used to run
the app. Neither profile may inherit an elevated or non-ConPTY connection; the
explicit profile must have no `source` or `updates`. The argumentless fallback bridge deliberately refuses to start a shell. Each real launch supplies fresh
owned nonce/profile/session arguments instead. Runtime validation checks these preconditions and rechecks known external
namespaces before every launch. It recognizes Terminal normalization only for
empty/null/omitted startup actions, removed invalid defaults command line, and
the exact observed PowerShell/Command Prompt inbox GUID/name/hidden-only records.
Unknown or changed entries remain refused; the HUD/default identity, explicit
fallback, connection/elevation and appearance checks remain required.
Unknown namespaces or discovery failure refuse launch. As with the accepted
adapter proof, this is not an atomic boundary against concurrent configuration
changes by programs under the same account.

Create an absolute JSON manifest with only the explicit executable and profile:

```json
{
  "terminal": "C:\\absolute-task-directory\\portable\\WindowsTerminal.exe",
  "profile": "{FRESH-PROFILE-GUID}"
}
```

The app reads this manifest and the synthetic settings; it never prepares or
installs the runtime. Keep source/settings baselines and raw diagnostics private
in the task directory. A launched bridge creates a fresh `TerminalHUD-session-*`
directory under long native TEMP, validates `WT_PROFILE_ID` and `WT_SESSION`,
starts system Windows PowerShell with `-NoLogo -NoProfile`, and publishes the
nonce/profile/session/PID proof. The adapter verifies a fresh window, exact
executable and retained creation ancestry before backend acquisition. The bridge
and interactive PowerShell remain independent of the tray owner's lifetime.
No session directory is reused or deleted automatically.

## Run and inspect

After source and runner review, reserve the desktop slot and announce the run.
From native PowerShell, with `$env:CARGO_TARGET_DIR` pointing to the native build:

```powershell
$cli = Join-Path $env:CARGO_TARGET_DIR 'debug\terminalhud.exe'
& $cli --development 'C:\absolute-task-directory\runtime.json' run
& $cli status
& $cli passive
& $cli visibility-toggle
& $cli visibility-toggle
& $cli interaction-toggle
& $cli quit
```

`run` initializes a fresh Active HUD before readiness is serviced. A duplicate
`run` contacts the existing same-user/session real owner. Later commands use
that real endpoint without reading a manifest. Explicit `--prototype` commands
use separate scopes and always report `mode=prototype`; real replies say
`mode=real`. Prototype callers cannot select the reserved real scope.

One native UI thread owns controller transitions, hotkey dispatch, periodic
observation and tray actions. A 200 ms timer and command-boundary refresh observe
closure/focus loss without spawning. Healthy observations preserve Hidden memory
and saved focus. Unexpected Active focus loss becomes Passive while preserving
the user's newly selected foreground. Observer failures report failed/unconfirmed
or recovered state rather than inferred success. Initial launch/transition
failures keep the tray alive and remain a failed status until another successful
state-changing command. CLI errors exit 2. Initial startup prints its private
native diagnostics-file path to stderr; acquisition logs record the owned session
and original style/visibility before mutation.

GUI spawning explicitly inherits only owned NUL/diagnostic handles, so a live
GUI cannot hold a caller's captured CLI stdout/stderr pipes open. The proof uses
private create-new capture files and bounded finite snapshots independently;
owner adoption requires its retained CLI's reported PID, direct parent and
creation interval, expected GUI image/SID/session, and a same-connection expected
PID status reply before any cleanup guard can send commands.

The selected `Win+Backtick` and `Win+Shift+Backtick` bindings are unchanged. Both
are registered on the UI thread through the backend API. A thread-only native
message hook queues their `WM_HOTKEY` messages even inside native modal loops;
it observes no keyboard input. `hotkey_error=0` in real mode means registration
succeeded. A nonzero Win32 code (commonly 1409 for an occupied chord) reports an
unavailable binding batch; diagnostics and the tray tooltip explain that tray
and IPC remain usable. Registration only unwinds bindings owned by this app.
The app never replaces or disables another application's binding. No alternate
binding or manual hotkey session is part of this demonstration.

Closing the dedicated Terminal leaves the tray running and status becomes
Absent. Absent `passive` and `status` do not spawn. Either toggle then launches
a fresh Active target with a new nonce. Quit restores the managed window's
original owned styles/visibility and leaves the bridge, interactive shell,
sessions and scrollback usable. A failed quit keeps the host alive. An
acquisition failure may leave its freshly launched terminal unmanaged and usable;
no arbitrary window or process is selected or terminated for cleanup.

## Bounds and evidence

The existing protected DACL, same-user/session peer checks, bounded 64-entry
queue, reentrancy guard, cancellation/storage lifetime rules, retained pipe and
reply receipt acknowledgement apply to both modes. The versioned reply is now
`THR2`, with an explicit mode and native hotkey failure code; the pipe namespace
is `TerminalHUD.shell.v2`. Request frames remain `THP1`.

Prototype deadlines remain 1500 ms per pipe operation and 2 s for a UI reply.
Real UI replies allow 20 s and client reply reads 22 s to cover at most 4 s of
namespace discovery plus a shared 6 s launch handshake/window-correlation budget
and backend work. Request reads, writes and ACKs retain 1500 ms bounds in both
modes. Startup allows 20 s plus a currently in-flight bounded IPC operation.
Real startup timeout preserves the owner and its restoration responsibility;
inspect `status` and diagnostics before another `run`. A timeout/disconnect after
a valid command is ambiguous; never blindly retry a toggle. No command retry or
persistent deduplication is implemented.

Portable suites cover lifecycle refresh, focus preservation, Hidden memory,
state recovery and honest real/prototype protocol/CLI routing. Unattended native
suites additionally exercise fail-closed runtime settings, thread-message hook
removal semantics, delayed real replies/ACK/disconnect ambiguity and prototype
scope isolation. They do not launch the real development HUD.

Desktop acceptance still requires the exact reviewed app/runner: native
presentation/focus, repeated IPC and tray transitions, same-session Hidden memory,
close/relaunch isolation and restored original state with an interactive shell
after Quit. Preserve unrelated settings/windows as read-only witnesses. Any test
sink must be ordinary non-topmost, with explicit HUD-above-sink checks. Do not
inject global keys, alter bindings/settings or obtain foreground through input.
If activation needs genuine user action, stop with the concrete scoped step.
The [accepted adapter evidence](windows-terminal-feasibility.md) establishes its
portable fixture behavior; it does not establish this app composition's desktop
acceptance. Record new actual outcomes and limitations before calling this slice
demonstrated.

## Explicit desktop runner

Build the runner together with the app binaries, without executing it:

```powershell
cargo build --locked -p terminalhud-app --bins --example app-slice-proof
```

After independent review of that exact source, runtime settings and compiled
artifact hashes, the desktop owner can approve this explicit invocation:

```powershell
& "$env:CARGO_TARGET_DIR\debug\examples\app-slice-proof.exe" 'C:\absolute-task-directory\runtime.json'
```

The outer operator must preserve external settings/fragment baselines, verify no
existing real endpoint/owned processes, apply a 90 s deadline, and record the
exact app/runner/runtime hashes before execution. The runner refuses an existing
real endpoint. It creates only its own ordinary blue sink, makes one checked
foreground request before launching the app, then drives real IPC and independently
reverifies the fresh target through the adapter's retained ancestry/session guard.
It checks Active/Hidden/Active and Passive/Hidden/Passive, closes only its verified
fresh synthetic session with no user work, observes Absent and no-spawn Passive,
and relaunches Active. Its single guarded click occurs at final Passive after
all activation checks, so it cannot be used to obtain foreground eligibility.
No cursor restoration or keyboard injection occurs. Quit must restore original
owned style bits/visibility and leave the exact replacement session alive.

A genuine foreground refusal ends the proof with failed/safe recovery evidence;
no grant, synthetic shortcut or focus workaround is attempted. Cleanup requests
only authenticated Quit from the exact retained real app owner. It never kills
the tray/terminal or closes a session after Quit. If ownership/readiness cannot
be established or Quit fails, the operator must inspect the preserved owner and
retry restoration through its normal IPC, with the exact recorded identity.
The original terminal closed during the lifecycle case contains only the fresh
synthetic interactive shell, with no injected or user work. The replacement shell
is deliberately left open after Quit for a genuine user command to demonstrate
post-Quit usability; process liveness alone is not a command-input proof.

The runner reuses the accepted adapter proof's ownership, fullscreen, sink,
relative-order and balanced-click code. Unrelated Terminal windows are read-only
style/visibility witnesses and external baseline checks remain the outer runner's
responsibility. A pass establishes this bounded native state/lifecycle slice.
It leaves physical selected-shortcut operation and actual post-Quit command input
unclaimed until separately observed, without a manual hotkey session.
