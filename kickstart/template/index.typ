// Editable whole-project entrypoint. The manifest records paths, not IDs.
#import "@preview/zettyp-core:0.1.0": eval
#import ".zettypst/zk/source.typ": load
#import ".zettypst/zk/graph.typ": build
#import ".zettypst/zk/relation.typ": derive

#let notes = load()
#let initial = build(notes.map(note => note.local))
#let graph-state = derive(initial.graph)

// Export observed identities and provenance without exporting full note bodies.
#eval.announce(<zk.notes>, graph-state
  .value
  .nodes
  .enumerate()
  .map(((index, node)) => (
    id: str(node.id),
    title: node.title,
    metadata: node.metadata,
    path: notes.at(index).path,
    origin: eval.inspect(graph-state.origin.nodes.at(index)),
  )))

#eval.announce(<zk.graph>, (
  value: graph-state.value,
  origin: (
    nodes: graph-state.origin.nodes.map(eval.inspect),
    edges: graph-state.origin.edges.map(eval.inspect),
  ),
))

#eval.announce(<zk.references.unclassified>, initial.unclassified.map(edge => (
  value: edge.value,
  origin: eval.inspect(edge.origin),
)))

// Use the target heading's title for ordinary references.
#show ref: it => {
  if it.element != none and it.element.func() == heading {
    link(it.target)[[#it.element.body]]
  } else {
    it
  }
}

#for note in notes {
  note.body
}
