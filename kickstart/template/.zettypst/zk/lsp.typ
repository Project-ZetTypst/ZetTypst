// Current graph state in, source-backed LSP announcements out.
#import "lsp/navigation.typ" as navigation
#import "lsp/definition.typ" as definition
#import "lsp/reference.typ" as reference
#import "lsp/hover.typ" as hover

#let consume(graph-state) = {
  let targets = navigation.targets(graph-state)
  definition.announce(targets)
  reference.announce(targets)
  hover.announce(targets)
}
