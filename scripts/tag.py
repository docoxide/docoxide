import argparse
import subprocess
import sys
import tomllib
from pathlib import Path

from rich.console import Console

ROOT = Path(__file__).resolve().parent.parent
TARGETS = ("rust", "python", "wasm")
VERSION_FILES: dict[str, Path] = {
    "rust": ROOT / "crates/docoxide/Cargo.toml",
    "python": ROOT / "bindings/python/Cargo.toml",
    "wasm": ROOT / "bindings/wasm/Cargo.toml",
}

console = Console(highlight=False, soft_wrap=True)
err_console = Console(stderr=True, highlight=False, soft_wrap=True)


def read_versions() -> dict[str, str]:
    return {target: tomllib.loads(path.read_text())["package"]["version"] for target, path in VERSION_FILES.items()}


def build_tags(targets: set[str], versions: dict[str, str]) -> list[str]:
    if targets:
        return [f"{t}-v{versions[t]}" for t in targets]

    distinct_versions = set(versions.values())

    if len(distinct_versions) > 1:
        raise RuntimeError(f"version mismatch across bindings: {versions}")

    return [f"all-v{next(iter(distinct_versions))}"]


def create_tag(tag: str, push: bool = False) -> None:
    subprocess.run(["git", "tag", tag], cwd=ROOT, check=True)

    if push:
        subprocess.run(["git", "push", "origin", tag], cwd=ROOT, check=True)


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Create release tags for docoxide bindings. Versions are read per binding from their respective manifests."
        ),
    )
    parser.add_argument(
        "targets",
        nargs="*",
        choices=TARGETS,
        metavar="TARGET",
        help=f"one or more of: {', '.join(TARGETS)} (default: creates a single all-v<version> tag)",
    )
    parser.add_argument("--push", action="store_true", help="push the tag to origin after creating it")
    parser.add_argument("--dry-run", action="store_true", help="show what would be done without creating the tag")
    args = parser.parse_args()

    targets = set(args.targets)

    try:
        versions = read_versions()
        tags = build_tags(targets, versions)
    except RuntimeError as e:
        err_console.print(f"[red]error:[/red] {e}")
        return 1

    for tag in tags:
        if args.dry_run:
            action = "create and push" if args.push else "create"
            console.print(f"dry-run: would {action} tag {tag}")
            continue

        create_tag(tag, args.push)
        console.print(f"created {tag}")

        if args.push:
            console.print(f"pushed {tag}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
