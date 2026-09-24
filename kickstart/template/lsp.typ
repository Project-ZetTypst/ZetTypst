// Evaluate editor semantics without rendering note bodies.
#import ".zettypst/zk/source.typ": load
#import ".zettypst/zk/graph.typ": build
#import ".zettypst/zk/relation.typ": evaluate
#import ".zettypst/zk/lsp.typ" as lsp

#let notes = load()
#let initial = build(notes.map(note => note.local))
#let result = evaluate(initial.graph)
// On failure, use the initial graph for navigation and suppress metadata hover.
#let graph-state = if result.value == none { initial.graph } else {
  result.value
}
#lsp.consume(graph-state, issues: result.issues)
