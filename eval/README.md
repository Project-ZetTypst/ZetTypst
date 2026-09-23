# zettyp-eval

Persistent Typst evaluation and source provenance, available as a Rust library and CLI. 
Collects `eval.announcement` values and externalizes `eval.inspect` source locations without interpreting note semantics or executing effects.

From the workspace root:

```sh
cargo build -p zettyp-eval --release --locked
cargo test --workspace --locked
cargo run -p zettyp-eval -- main.typ --root . --input key=value
cargo run -p zettyp-eval -- serve --root . --socket /tmp/zettyp-eval.sock
```

Licensed under [MIT](../LICENSE).
