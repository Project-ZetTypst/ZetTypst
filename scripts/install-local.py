#!/usr/bin/env python3
"""Install the workspace's Typst packages locally. Requires Python 3.11+."""

import argparse
from pathlib import Path
import re
import shutil
import tomllib


def install(package_root: Path, package_path: Path, *, link: bool = False) -> None:
    package_root = package_root.resolve()
    manifest = package_root / "typst.toml"
    config = tomllib.loads(manifest.read_text(encoding="utf-8"))
    package = config["package"]
    name, version = package["name"], package["version"]
    if not re.fullmatch(r"[a-z][a-z0-9-]*", name):
        raise ValueError(f"Invalid package name: {name!r}")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise ValueError(f"Invalid package version: {version!r}")

    # Copy the public package files, plus the template directory when declared.
    files = [manifest, package_root / "LICENSE", package_root / "README.md"]
    files.extend(sorted(package_root.glob("*.typ")))
    if package_root / package["entrypoint"] not in files:
        raise ValueError("Package entrypoint must be a top-level .typ file")
    for source in files:
        if not source.is_file():
            raise FileNotFoundError(source)

    template = config.get("template")
    template_path = None
    if template is not None:
        template_path = Path(template["path"])
        entrypoint = Path(template["entrypoint"])
        for path in (template_path, entrypoint):
            if path.is_absolute() or not path.parts or ".." in path.parts:
                raise ValueError(f"Invalid template path: {path}")
        template_root = package_root / template_path
        if not template_root.is_dir():
            raise FileNotFoundError(template_root)
        if template_root.is_symlink() or any(
            path.is_symlink() for path in template_root.rglob("*")
        ):
            raise ValueError(f"Template must not contain symlinks: {template_root}")
        if not (template_root / entrypoint).is_file():
            raise FileNotFoundError(template_root / entrypoint)

    parent = package_path / "preview" / name
    for directory in (package_path, package_path / "preview", parent):
        if directory.is_symlink():
            raise ValueError(f"Refusing to install through a symlink: {directory}")
    destination = parent / version
    if destination.is_symlink():
        if destination.resolve() != package_root:
            raise ValueError(f"Refusing to replace an unrelated symlink: {destination}")
        if not link:
            destination.unlink()  # Never rmtree through a package-source link.
    elif destination.exists():
        shutil.rmtree(destination)

    if link:
        parent.mkdir(parents=True, exist_ok=True)
        if not destination.is_symlink():
            destination.symlink_to(package_root, target_is_directory=True)
    else:
        destination.mkdir(parents=True)
        for source in files:
            shutil.copy2(source, destination / source.name)
        if template_path is not None:
            shutil.copytree(package_root / template_path, destination / template_path)

    mode = "Linked" if link else "Copied"
    print(f"{mode} @preview/{name}:{version} to {destination}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--link",
        action="store_true",
        help="Link package sources instead of copying them.",
    )
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    package_path = root / ".dev" / "packages"
    for directory in ("core", "lsp/typst", "kickstart"):
        install(root / directory, package_path, link=args.link)
    print(f"Package path: {package_path}")


if __name__ == "__main__":
    main()
