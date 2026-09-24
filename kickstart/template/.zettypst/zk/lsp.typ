// Current graph state in, source-backed LSP announcements out.
#import "lsp/navigation.typ" as navigation
#import "lsp/definition.typ" as definition
#import "lsp/reference.typ" as reference
#import "lsp/hover.typ" as hover
#import "lsp/diagnostic.typ" as diagnostic

#let consume(graph-state, issues: ()) = {
  let targets = navigation.targets(graph-state)
  definition.announce(targets)
  reference.announce(targets)
  // Only present metadata when relation inference succeeded.
  if issues.len() == 0 {
    hover.announce(targets)
  }
  diagnostic.announce(graph-state, issues: issues)
}
