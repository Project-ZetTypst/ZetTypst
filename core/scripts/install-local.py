#!/usr/bin/env python3
"""Install the flat Core package locally. Requires Python 3.11+."""

from pathlib import Path
import re
import shutil
import tomllib


def main() -> None:
    core = Path(__file__).resolve().parents[1]
    manifest = core / "typst.toml"
    package = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]
    name, version = package["name"], package["version"]
    if not re.fullmatch(r"[a-z][a-z0-9-]*", name):
        raise ValueError(f"Invalid package name: {name!r}")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError(f"Invalid package version: {version!r}")

    # Explicitly package the current flat layout, not development tooling.
    files = [manifest, core / "LICENSE", core / "README.md"]
    files.extend(sorted(core.glob("*.typ")))
    if core / package["entrypoint"] not in files:
        raise ValueError("Package entrypoint must be a top-level .typ file")
    for source in files:
        if not source.is_file():
            raise FileNotFoundError(source)

    package_path = core.parent / ".dev" / "packages"
    destination = package_path / "preview" / name / version
    if destination.is_symlink():
        raise ValueError(f"Refusing to replace a symlink: {destination}")
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    for source in files:
        shutil.copy2(source, destination / source.name)

    print(f"Installed @preview/{name}:{version} to {destination}")
    print(f"Package path: {package_path}")


if __name__ == "__main__":
    main()
