# zettyp-eval

Persistent Typst evaluation and source provenance, available as a Rust library and CLI. 

From the workspace root:

```sh
cargo build -p zettyp-eval --release --locked
cargo test --workspace --locked
cargo run -p zettyp-eval -- main.typ --root . --input key=value
cargo run -p zettyp-eval -- serve --root . --socket /tmp/zettyp-eval.sock
```

Licensed under [MIT](../LICENSE).
