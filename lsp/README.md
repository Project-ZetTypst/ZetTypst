# zettyp-lsp

A stdio LSP adapter for Typst announcements, backed by one persistent `zettyp-eval` runtime.

From the workspace root:

```sh
cargo build -p zettyp-lsp --release --locked
cargo run -p zettyp-lsp -- --root .
```

[MIT](../LICENSE).
