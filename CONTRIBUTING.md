# Contributing

## Local environment

Use Python 3.11+, Typst, and Bash or Zsh. Rust development uses the stable toolchain specified in `rust-toolchain.toml`.

From the workspace root:

```sh
source scripts/dev.sh
```

This links Core, the Typst LSP wrapper, and Kickstart into `.dev/packages/preview/` and sets `TYPST_PACKAGE_PATH` for the current shell. Typst, `zettyp-eval`, and `zettyp-lsp` then resolve the usual `@preview` imports without extra package-path flags. No global package installation or shell configuration is changed.

Run it again in each new shell, or after changing a package version or moving the repository. Source changes are immediately visible through the links; running processes may still need a refresh or restart.

To check copied packages instead of live links:

```sh
python3 scripts/install-local.py
```

This replaces the current-version links with copies without modifying their source directories. Rerun after source changes; `source scripts/dev.sh` switches back to links. Link mode requires permission to create directory symlinks.

## Typst and Kickstart

Edit the template directly during development:

```sh
typst watch --root kickstart/template kickstart/template/index.typ .dev/kickstart.pdf
```

To check the initialized project, choose a directory that does not exist:

```sh
typst init @preview/zettyp-kickstart:0.1.0 .dev/my-notes
cd .dev/my-notes
typst watch index.typ
```

`typst init` copies the template. Later edits to `kickstart/template/` do not update that generated project. Return to the workspace root for the commands below.

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

For Kickstart, configure Tinymist's project root as `kickstart/template/` and its main entry as `index.typ`. The local `zettyp-lsp` binary is `target/release/zettyp-lsp`; pass `--root` with that project directory and use initialization options `{"entry": "index.typ"}`. Tinymist's usual language features and the announcement-driven LSP are separate services. Kickstart's LSP policies are not wired up yet.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
typst compile --root kickstart/template kickstart/template/index.typ .dev/kickstart.pdf
git diff --check
```
