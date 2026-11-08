# Contributing

Follow [the development workflow](docs/development.md) to choose a task, create
a branch, run checks, submit a PR, and release. Humans and agents follow the
same process. Agents also read [AGENTS.md](AGENTS.md).

Run the workspace checks before handoff:

```bash
python3 scripts/check_repository.py
python3 scripts/check_workspace.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
git diff --check
```

Python 3.11 or later is required for the repository check. It validates the
Ratpack configuration, role profile syntax, and local Markdown link targets.
The `check` GitHub Actions job requires repository checks and native Ubuntu/Windows
Rust checks for pull requests and pushes to `main` and `devel`. Record validation
evidence in the linked Linear task.

Follow [the build instructions](docs/building.md) for native Windows execution
and artifact paths. Add meaningful behavior tests as components are implemented.
Product acceptance is defined by the
[requirements](docs/requirements.md) and
[validation plan](docs/validation-plan.md).
