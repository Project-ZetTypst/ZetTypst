# Contributing

## Local environment

Use Python 3.11+, Typst, and Bash or Zsh. Rust development uses the stable toolchain specified in `rust-toolchain.toml`.

From the workspace root:

```sh
source scripts/dev.sh
```

This assembles local packages in `.dev/packages/preview/`, links their source files, and sets `TYPST_PACKAGE_PATH` for the current shell. Core includes the Kickstart template; the LSP and Site wrappers remain separate packages. Typst, `zettyp-eval`, and `zettyp-lsp` then resolve the usual `@preview` imports without extra package-path flags. No global package installation or shell configuration is changed.

Run it again in each new shell, or after changing a package version or moving the repository. Source changes are immediately visible through the links; running processes may still need a refresh or restart.

To check copied packages instead of live links:

```sh
python3 scripts/install-local.py
```

This replaces the current-version links with copies without modifying their source directories. Rerun after source changes; `source scripts/dev.sh` switches back to links. Link mode requires permission to create file symlinks.

## Typst and Kickstart

Edit the template directly during development:

```sh
typst watch --root kickstart/template kickstart/template/index.typ .dev/kickstart.pdf
```

Core and Kickstart remain separate source directories but are distributed together as `zettyp-core`. To assemble release directories with copied files:

```sh
python3 scripts/package_typst.py
```

The output is `.dev/dist/preview/`, containing Core with its template and the LSP wrapper. Generated files should not be edited or committed.

## Rust tools

```sh
cargo build --workspace --release --locked
cargo run -p zettyp-eval -- kickstart/template/index.typ --root kickstart/template
```

For a persistent evaluator (Unix only):

```sh
cargo run -p zettyp-eval -- serve --root kickstart/template --socket .dev/zettyp-eval.sock
```

## Editor setup

Use the same absolute `.dev/packages` path in all tools. Launch the editor from the activated shell or configure that path explicitly in its project settings. An already-running editor does not inherit later shell changes.

For Kickstart, configure Tinymist's project root as `kickstart/template/` and its main entry as `index.typ`. The local `zettyp-lsp` binary is `target/release/zettyp-lsp`; pass `--root` with that project directory and use initialization options `{"entry": "lsp.typ"}`. Tinymist's usual language features and the announcement-driven LSP are separate services. Kickstart's `lsp.typ` publishes Definition, References, Hover, and Diagnostics without rendering note bodies; project configuration and policies live in `.zettypst/lib.typ`.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
typst compile --root kickstart/template kickstart/template/index.typ .dev/kickstart.pdf
git diff --check
```
