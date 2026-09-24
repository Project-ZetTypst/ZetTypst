// Evaluate editor semantics without rendering note bodies.
#import ".zettypst/zk/source.typ": load
#import ".zettypst/zk/graph.typ": build
#import ".zettypst/zk/lsp.typ" as lsp

#let notes = load()
#let initial = build(notes.map(note => note.local))
#lsp.consume(initial.graph)
