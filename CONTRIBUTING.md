# Contributing

Follow [the development workflow](docs/development.md) to choose a task, create
a branch, run checks, submit a PR, and release. Humans and agents follow the
same process. Agents also read [AGENTS.md](AGENTS.md).

The repository currently contains documentation and contributor tooling. Run:

```bash
python3 scripts/check_repository.py
git diff --check
```

Python 3.11 or later is required for the repository check. It validates the
Ratpack configuration, role profile syntax, and local Markdown link targets.
The `check` GitHub Actions job runs these checks for pull requests and pushes to
`main` and `devel`. Record validation evidence in the linked Linear task.

Rust formatting, Clippy, Cargo tests, and Windows integration checks must be
added with the first implementation. Product acceptance is defined by the
[requirements](docs/requirements.md) and
[validation plan](docs/validation-plan.md).
