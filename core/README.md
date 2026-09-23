# ZetTypst Core

The minimal, policy-free core for expressing knowledge semantics in Typst.
Documents and their semantic rules share one language; external tools consume the results via announcements rather than reinterpret the notes.

Import:

```typ
#import "@preview/zettyp-core:0.1.0": eval, graph, node
```

Local development:

```sh
python3 core/scripts/install-local.py
```

Use `--package-path .dev/packages` with Typst or `zettyp-eval`; configure the editor likewise.

[MIT License](LICENSE).
