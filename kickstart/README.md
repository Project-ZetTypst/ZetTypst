# zettyp-kickstart

An example configuration for ZetTypst.

Start:

```sh
python3 scripts/install-local.py
mkdir -p .dev/packages/preview/zettyp-kickstart/0.1.0
cp -R kickstart/. .dev/packages/preview/zettyp-kickstart/0.1.0/
typst init --package-path .dev/packages @preview/zettyp-kickstart:0.1.0 my-notes
cd my-notes
typst watch --package-path ../.dev/packages index.typ
```

[MIT](LICENSE).
