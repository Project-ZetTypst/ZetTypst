// Adapt relation issues to LSP without duplicating graph validation rules.
#import "@preview/zettyp-lsp:0.1.0" as lsp

#let announce(graph-state, issues: ()) = {
  // Keep empty publications so repaired nodes clear their previous diagnostics.
  for origin in graph-state.origin.nodes {
    lsp.announce(lsp.effect-kinds.publish-diagnostics, lsp.publish-diagnostics(
      document: origin,
      diagnostics: (),
    ))
  }
  for issue in issues {
    for origin in issue.origins {
      // Publish against the occurrence's own file, including imported content.
      lsp.announce(
        lsp.effect-kinds.publish-diagnostics,
        lsp.publish-diagnostics(
          document: origin,
          diagnostics: (
            lsp.diagnostic(
              origin: origin,
              message: issue.message,
              severity: lsp.severity.error,
              code: issue.code,
              source: "zettypst",
            ),
          ),
        ),
      )
    }
  }
}
