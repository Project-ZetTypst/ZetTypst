// Keep reference occurrences separate from the optional declaration result.
#import "@preview/zettyp-lsp:0.1.0" as lsp

#let announce(targets) = {
  for target in targets {
    lsp.announce(lsp.effect-kinds.references, lsp.references(
      applies-to: target.origin,
      targets: target.references,
      declarations: (target.definition,),
    ))
  }
}
