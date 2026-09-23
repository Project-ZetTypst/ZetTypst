#!/usr/bin/env python3
"""Install the workspace's Typst packages locally. Requires Python 3.11+."""

from pathlib import Path
import re
import shutil
import tomllib


def install(package_root: Path, package_path: Path) -> None:
    manifest = package_root / "typst.toml"
    package = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]
    name, version = package["name"], package["version"]
    if not re.fullmatch(r"[a-z][a-z0-9-]*", name):
        raise ValueError(f"Invalid package name: {name!r}")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError(f"Invalid package version: {version!r}")

    # Explicitly package the current flat layout, not development tooling.
    files = [manifest, package_root / "LICENSE", package_root / "README.md"]
    files.extend(sorted(package_root.glob("*.typ")))
    if package_root / package["entrypoint"] not in files:
        raise ValueError("Package entrypoint must be a top-level .typ file")
    for source in files:
        if not source.is_file():
            raise FileNotFoundError(source)

    destination = package_path / "preview" / name / version
    if destination.is_symlink():
        raise ValueError(f"Refusing to replace a symlink: {destination}")
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    for source in files:
        shutil.copy2(source, destination / source.name)

    print(f"Installed @preview/{name}:{version} to {destination}")


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    package_path = root / ".dev" / "packages"
    for directory in ("core", "lsp/typst"):
        install(root / directory, package_path)
    print(f"Package path: {package_path}")


if __name__ == "__main__":
    main()
