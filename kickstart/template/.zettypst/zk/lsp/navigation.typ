// Resolve source occurrences against the current graph, not file names.
#import "../graph-node.typ": relations

#let targets(graph-state) = {
  let graph = graph-state.value
  let origins = graph-state.origin
  let nodes = graph.nodes.enumerate().map(((index, node)) => (
    node: node,
    definition: origins.nodes.at(index),
    references: graph.edges
      .enumerate()
      .filter(((i, edge)) => edge.relation == relations.ref and edge.target == node.id)
      .map(((i, edge)) => origins.edges.at(i)),
  ))

  let targets = ()
  // References precede declarations so a ref inside a heading takes priority
  // over that heading's larger source range when the host selects a result.
  for (index, edge) in graph.edges.enumerate() {
    if edge.relation == relations.ref {
      let target = nodes.find(item => item.node.id == edge.target)
      if target != none {
        targets.push(target + (origin: origins.edges.at(index),))
      }
    }
  }
  for target in nodes {
    targets.push(target + (origin: target.definition,))
  }
  targets
}
