/// This module organizes node and edge values into a globally valid attributed
/// graph, and organizes their source-anchored atomic states into one graph-level
/// state. It defines two structural dictionary types:
///
/// ```typ
/// Graph = (
///   nodes: array<Node>,
///   edges: array<Edge>,
/// )
/// GraphState = (
///   value: Graph,
///   origin: (
///     nodes: array<content>,
///     edges: array<content>,
///   ),
/// )
/// ```
///
/// Node IDs are globally unique, and every edge endpoint belongs to the graph.
/// Arrays preserve their input order: parallel edge occurrences remain distinct,
/// and every graph value stays positionally aligned with its source origin.

#let valid-node(value) = (
  type(value) == dictionary
    and "id" in value
    and "title" in value
    and "metadata" in value
    and type(value.id) == label
    and type(value.title) == content
    and type(value.metadata) == dictionary
)

#let valid-edge(value) = (
  type(value) == dictionary
    and "source" in value
    and "relation" in value
    and "target" in value
    and type(value.source) == label
    and type(value.relation) == label
    and type(value.target) == label
)

#let valid-state(value, valid-value) = (
  type(value) == dictionary
    and "value" in value
    and "origin" in value
    and valid-value(value.value)
    and type(value.origin) == content
)

/// Construct a globally valid attributed directed multigraph.
///
/// Node IDs must be unique, and every edge source and target must identify a
/// node in the graph. Node and edge arrays retain their input order; parallel
/// edge occurrences are not deduplicated.
///
/// - nodes (array): The graph's semantic `Node` values.
/// - edges (array): The graph's semantic `Edge` occurrences.
/// -> dictionary
#let graph(nodes: (), edges: ()) = {
  if type(nodes) != array {
    panic("graph nodes must be an array")
  }
  if type(edges) != array {
    panic("graph edges must be an array")
  }

  let ids = ()
  for node in nodes {
    if not valid-node(node) {
      panic("graph nodes must contain Node values")
    }
    if node.id in ids {
      panic("duplicate graph node ID: " + repr(node.id))
    }
    ids.push(node.id)
  }

  for edge in edges {
    if not valid-edge(edge) {
      panic("graph edges must contain Edge values")
    }
    if edge.source not in ids {
      panic(
        "graph edge source does not identify a node: " + repr(edge.source),
      )
    }
    if edge.target not in ids {
      panic(
        "graph edge target does not identify a node: " + repr(edge.target),
      )
    }
  }

  (nodes: nodes, edges: edges)
}

/// Organize source-anchored node and edge states into one `GraphState`.
///
/// The returned state's `value` is a globally validated `Graph`. Its graph-
/// shaped `origin` contains node and edge source owners in the same order as
/// their corresponding values.
///
/// - nodes (array): Atomic `State<Node>` values.
/// - edges (array): Atomic `State<Edge>` occurrences.
/// -> dictionary
#let state(nodes: (), edges: ()) = {
  if type(nodes) != array {
    panic("graph node states must be an array")
  }
  if type(edges) != array {
    panic("graph edge states must be an array")
  }
  if not nodes.all(value => valid-state(value, valid-node)) {
    panic("graph node states must contain State<Node> values")
  }
  if not edges.all(value => valid-state(value, valid-edge)) {
    panic("graph edge states must contain State<Edge> values")
  }

  (
    value: graph(
      nodes: nodes.map(state => state.value),
      edges: edges.map(state => state.value),
    ),
    origin: (
      nodes: nodes.map(state => state.origin),
      edges: edges.map(state => state.origin),
    ),
  )
}
