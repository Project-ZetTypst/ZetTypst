// Aggregate node-local observations into a source-backed graph.
#import "@preview/zettyp-core:0.1.0": graph, node

#let build(observations) = {
  let locals = observations.map(local => node.local-state(
    node: local.node,
    outgoing: local.outgoing,
  ))
  let nodes = locals.map(local => local.node)
  let ids = nodes.map(state => state.value.id)
  let edges = ()
  let unclassified = ()
  for local in locals {
    for edge in local.outgoing {
      if edge.value.target in ids {
        edges.push(edge)
      } else {
        unclassified.push(edge)
      }
    }
  }
  (
    graph: graph.state(nodes: nodes, edges: edges),
    // These may be anchors, bibliography references, or missing note targets.
    // Keep every occurrence for later policy; absence from the graph is not
    // itself a broken-link diagnosis.
    unclassified: unclassified,
  )
}
