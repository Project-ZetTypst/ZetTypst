// Navigate from a node declaration or reference to the original declaration.
#import "@preview/zettyp-lsp:0.1.0" as lsp

#let announce(targets) = {
  for target in targets {
    lsp.announce(lsp.effect-kinds.definition, lsp.definition(
      applies-to: target.origin,
      target: target.definition,
    ))
  }
}
