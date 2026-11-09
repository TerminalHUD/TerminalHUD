# Building and validating the workspace

## Toolchains

Use stable Rust with rustfmt and Clippy. `rust-toolchain.toml` selects those
components for this checkout; it does not change the global Rust default. The
root is a virtual Cargo workspace using edition 2024 and resolver 3. Commit its
shared `Cargo.lock` and use `--locked` in checks. The package version is a
development version, not a release declaration.

Windows development uses native MSVC Rust, Microsoft C++ Build Tools, and a
Windows SDK. See [Microsoft's setup instructions](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup)
and [Rust's Windows target requirements](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html).
Install missing system components deliberately. WSL Rust is useful
for portable checks but does not establish native Windows behavior.

## Crate and directory map

| Location | Responsibility | Runtime workspace dependencies |
| --- | --- | --- |
| `src/hudswitch-core` | Portable generic state/controller contracts | None |
| `src/hudswitch-windows` | Win32 backend | HudSwitch core |
| `src/terminalhud-core` | Portable product orchestration and adapter contracts | HudSwitch core |
| `src/terminal-adapters/windows-terminal` | Dedicated Windows Terminal integration | Both portable cores |
| `src/terminalhud-app` | Composition root, tray/CLI/IPC application | Product core; Windows backend and adapter on Windows |
| `tests/hudswitch-core-tests` | Portable black-box and fake-backend suites | Registered with their owning crate as tests are added |
| `tests/hudswitch-backend-tests` | Backend integration suites | Generic window harness, independent of terminals |
| `tests/hudswitch-test-window` | Generic harness and native build probe | Probe registered as a backend Cargo example |
| `tests/terminalhud-tests` | Product and adapter suites | Registered with their owning crate as tests are added |
| `integrations/shells` | PowerShell and Bash integration sources | Same-user application commands |
| `packaging/windows` | Per-user Windows packaging resources | Application build artifacts |

This is the long-term architecture in [architecture.md](architecture.md), with
Windows components implemented first. Future platform backends and adapters fit
alongside these paths. The application shell provides an explicit simulated prototype and a real
[Windows development slice](windows-development-slice.md). The prototype creates
no terminal window. See [application shell](application-shell.md) for its commands
and security boundary.

`python3 scripts/check_workspace.py` obtains Cargo metadata and validates the
reviewed crate locations and runtime dependency direction. Platform-specific
dependencies are excluded from the portable cores. New architectural boundaries
must update that check deliberately. Portable controller, product command,
protocol, native backend and shell IPC suites are registered with their owning
crates. Visible desktop suites are explicitly ignored during unattended tests.

## Portable checks from WSL or Linux

From the repository root (fetch the locked dependencies once before the offline
workspace metadata check on a fresh checkout):

```bash
cargo fetch --locked
python3 scripts/check_repository.py
python3 scripts/check_workspace.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --all-targets --locked
cargo test --workspace --locked
git diff --check
```

Cargo normally places outputs in `target/` at the workspace root. If the
checkout is on a small temporary filesystem, set `CARGO_TARGET_DIR` to a
checkout-specific directory on a filesystem with enough space. This changes only
the current process environment; do not share a build directory between Linux
and native Windows. See [Cargo workspace behavior](https://doc.rust-lang.org/cargo/reference/workspaces.html).

## Native Windows checks from WSL

Windows tools are separate from WSL tools. From the repository root in WSL,
obtain the Windows-visible source path and open native Windows PowerShell:

```bash
wslpath -w "$PWD"
powershell.exe -NoProfile
```

In that PowerShell session, change to the path printed by `wslpath` (substitute
your actual distribution and checkout path below). Keep native outputs on the
Windows filesystem, separate from Linux `target/`:

```powershell
Set-Location '\\wsl.localhost\Ubuntu\path\to\terminalhud'
$env:CARGO_TARGET_DIR = Join-Path $env:TEMP "terminalhud-target"
rustc -vV
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --all-targets --locked
cargo test --workspace --locked
cargo run --locked -p hudswitch-windows --example toolchain-probe
```

Confirm that `rustc -vV` reports a native `*-pc-windows-msvc` host. Check each
command's exit status (`$LASTEXITCODE`) and stop if a check fails. The environment
assignment affects only this PowerShell process. Choose a distinct native output
path for concurrent worktrees. Cargo incremental compilation locks do not work
when the Windows target directory is itself on a WSL UNC path; native `%TEMP%`
outputs work while source files remain in WSL.

These direct commands work without changing Windows script-execution policy.
Some policies treat scripts on WSL UNC paths as remote and reject unsigned
`.ps1` files. Preserve those policies; use the direct native Cargo commands.
Nothing here installs software or changes global defaults.

The probe executable is at
`%TEMP%\terminalhud-target\debug\examples\toolchain-probe.exe` with the output
path above. It prints the compiled OS, architecture, and process ID. A Windows
run must report `os=windows`; Linux execution is not Windows validation.
This is native process-execution evidence only. Interactive GUI validation of
focus, hotkeys, click-through, and the generic test window belongs to the backend
and application tasks, and must run in the actual Windows desktop session.

## Continuous integration

The workflow builds and tests on Ubuntu and Windows using native stable Rust.
Both jobs check the architecture map, formatting, Clippy, all-target builds,
workspace tests, and the toolchain probe. Repository/documentation checks run
separately. The required `check` job aggregates their results, so a failing or
skipped prerequisite cannot produce a passing required check.

## Tray prototype and console CLI

Build both application binaries together from native PowerShell:

```powershell
cargo build --locked -p terminalhud-app --bins
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype run
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype status
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype interaction-toggle
& "$env:CARGO_TARGET_DIR\debug\terminalhud.exe" --prototype quit
```

`terminalhud.exe` is the synchronous console CLI: it prints a reply and exits 0,
or prints an error to stderr and exits 2. `run` starts its sibling
`terminalhud-shell.exe`, waits for an authenticated controller status reply, and
returns. The shell binary uses the Windows GUI subsystem, owns the tray and
message loop, and creates no console or visible main window. Ship the two
binaries together. The commands above use `--prototype` and simulate HUD state. See the
[real development guide](windows-development-slice.md) for the isolated native
runtime manifest and real commands. The production path targets installed Windows
Terminal and has a separate proposed, explicitly approved profile setup; see the
[application shell guide](application-shell.md).
A native PowerShell call made through `powershell.exe -NoProfile` from WSL also
waits for the console CLI and preserves its captured stdout and exit code.

The unattended native shell suite is included in `cargo test --workspace`.
Run the visible shell proof only in a real desktop session with the desktop
slot reserved and no other interactive test running:

```powershell
cargo test --locked -p terminalhud-app --test shell_ipc tray_menu_and_ipc_remain_responsive_in_modal_loop -- --ignored --nocapture --test-threads=1
```

The optional bounded event-loop comparison is separate from the production
runtime. Each invocation briefly creates its own tray icon and exits:

```powershell
$loopReports = Join-Path $env:TEMP 'terminalhud-ter-10-loop-reports'
New-Item -ItemType Directory -Force -Path $loopReports | Out-Null
cargo run --locked -p terminalhud-app --features loop-comparison --example shell-loop-probe -- native "$loopReports\native.txt"
cargo run --locked -p terminalhud-app --features loop-comparison --example shell-loop-probe -- tao "$loopReports\tao.txt"
```

Each invocation requires both the loop mode and a report-file path. Keep these
task-owned reports on the native Windows filesystem, separate from the WSL UNC
source path, and choose a distinct report directory for concurrent worktrees.

See [the evaluation](ui-toolkit-evaluation.md) for the measured selection and
[the shell guide](application-shell.md) for prototype limits and IPC behavior.
