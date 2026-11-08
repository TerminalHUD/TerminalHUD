#!/usr/bin/env python3
"""Check contributor configuration and local Markdown link targets; Python 3.11+."""

import json
from pathlib import Path
import re
import sys
import tomllib
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]


def main():
    errors = []
    try:
        config = json.loads((ROOT / "ratpack.json").read_text(encoding="utf-8"))
        if config.get("schema_version") != 1:
            errors.append("ratpack.json: expected schema_version 1")
        count = config.get("max_implementation_agents")
        if "max_implementation_agents" not in config:
            errors.append("ratpack.json: missing implementation worker count")
        elif count is not None and (type(count) is not int or count < 0):
            errors.append("ratpack.json: worker count must be nonnegative or null")
        names = set()
        for role in ("coordinator", "implementor", "reviewer", "cleanup", "workflow"):
            path = ROOT / ".codex" / "agents" / f"ratpack_{role}.toml"
            profile = tomllib.loads(path.read_text(encoding="utf-8"))
            if profile.get("name") != f"ratpack_{role}":
                errors.append(f"{path.relative_to(ROOT)}: unexpected profile name")
            for field in ("name", "description", "developer_instructions"):
                if not isinstance(profile.get(field), str) or not profile[field].strip():
                    errors.append(f"{path.relative_to(ROOT)}: missing {field}")
            if profile.get("name") in names:
                errors.append(f"{path.relative_to(ROOT)}: duplicate profile name")
            names.add(profile.get("name"))
    except (OSError, ValueError, AttributeError, TypeError) as error:
        errors.append(f"Contributor configuration: {error}")

    documents = [*ROOT.glob("*.md"), *ROOT.joinpath("docs").rglob("*.md")]
    for document in documents:
        text = document.read_text(encoding="utf-8")
        # The project uses inline Markdown links; skip fenced code examples.
        text = re.sub(r"```.*?```", "", text, flags=re.DOTALL)
        for match in re.finditer(r"\[[^\]]*\]\(([^\s)]+)\)", text):
            target = urlsplit(match.group(1).strip("<>"))
            if target.scheme or target.netloc or not target.path:
                continue
            path = (document.parent / unquote(target.path)).resolve()
            if not path.is_relative_to(ROOT) or not path.exists():
                errors.append(f"{document.relative_to(ROOT)}: missing local target {target.path}")

    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print(f"Repository checks passed: configuration, five profiles, {len(documents)} documents")
    return 0


if __name__ == "__main__":
    sys.exit(main())
