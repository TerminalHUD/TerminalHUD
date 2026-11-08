#!/usr/bin/env python3
"""Validate the foundation's Cargo layout and architectural dependency direction."""

import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
# Deliberate new workspace boundaries must update this map during review.
PACKAGES = {
    "hudswitch-core": ("src/hudswitch-core", set()),
    "hudswitch-windows": ("src/hudswitch-windows", {"hudswitch-core"}),
    "terminalhud-core": ("src/terminalhud-core", {"hudswitch-core"}),
    "terminalhud-windows-terminal": (
        "src/terminal-adapters/windows-terminal", {"hudswitch-core", "terminalhud-core"}
    ),
    "terminalhud-app": (
        "src/terminalhud-app",
        {"terminalhud-core", "hudswitch-windows", "terminalhud-windows-terminal"},
    ),
}


def main():
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked", "--offline"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if result.returncode:
        print(result.stderr, file=sys.stderr)
        return result.returncode
    metadata = json.loads(result.stdout)
    packages = {p["name"]: p for p in metadata["packages"]}
    errors = []
    if set(packages) != set(PACKAGES):
        errors.append("Workspace packages differ from the reviewed architecture map")
    for name, (relative, allowed) in PACKAGES.items():
        package = packages.get(name)
        if package is None:
            errors.append(f"Missing workspace package: {name}")
            continue
        if Path(package["manifest_path"]).resolve() != ROOT / relative / "Cargo.toml":
            errors.append(f"{name}: expected manifest at {relative}/Cargo.toml")
        for dependency in package["dependencies"]:
            # Test-only dependencies may compose layers without changing the runtime graph.
            if dependency["kind"] == "dev":
                continue
            if dependency["name"] in packages and dependency["name"] not in allowed:
                errors.append(f"{name}: forbidden dependency on {dependency['name']}")
            if name in ("hudswitch-core", "terminalhud-core") and dependency["target"]:
                errors.append(f"{name}: portable core has a platform-specific dependency")
    for relative in (
        "tests/hudswitch-core-tests", "tests/hudswitch-backend-tests",
        "tests/hudswitch-test-window", "tests/terminalhud-tests",
        "integrations/shells", "packaging/windows", "docs",
    ):
        if not (ROOT / relative).is_dir():
            errors.append(f"Missing long-term monorepo location: {relative}")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print("Workspace architecture checks passed: five crates and long-term directory boundaries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
