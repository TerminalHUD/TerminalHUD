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
alongside these paths. The foundation creates crate boundaries, not the runtime
state machine, window backend, tray, IPC, or terminal adapter behavior. The app
binary is currently an empty composition-root scaffold and provides no HUD.

`python3 scripts/check_workspace.py` obtains Cargo metadata and validates the
reviewed crate locations and runtime dependency direction. Platform-specific
dependencies are excluded from the portable cores. New architectural boundaries
must update that check deliberately. No behavioral Rust tests exist yet;
`cargo test` currently proves test-harness compilation/linking and execution.
Real suites belong in their designated test directories and must be registered
with the owning crate so workspace tests discover them.

## Portable checks from WSL or Linux

From the repository root:

```bash
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
