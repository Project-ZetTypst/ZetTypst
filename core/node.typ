/// This module models one node's semantic value, its outgoing edges, and the
/// source-backed state slots that carry them. It defines four structural
/// dictionary types:
///
/// ```typ
/// Node = (id: label, title: content, metadata: dictionary)
/// Edge = (source: label, relation: label, target: label)
/// State<T> = (value: T, origin: content)
/// LocalNodeState = (
///   node: State<Node>,
///   outgoing: array<State<Edge>>,
/// )
/// ```
///
/// `State<T>` carries a current semantic value and its stable source origin.
/// `LocalNodeState` groups one node state with its outgoing edge states. This
/// core validates node-local structure: every outgoing edge must use the
/// node state's ID as its source.

/// Construct a semantic node value.
///
/// - id (label): The node's Typst-native identity.
/// - title (content): The evaluated node title.
/// - metadata (dictionary): The node's open metadata payload.
/// -> dictionary
#let node(id: none, title: none, metadata: (:)) = {
  if type(id) != label {
    panic("node ID must be a label")
  }
  if type(title) != content {
    panic("node title must be content")
  }
  if type(metadata) != dictionary {
    panic("node metadata must be a dictionary")
  }
  (id: id, title: title, metadata: metadata)
}

/// Construct one directed edge value.
///
/// - source (label): The source node identity.
/// - relation (label): The open semantic relation identity.
/// - target (label): The target node identity.
/// -> dictionary
#let edge(source: none, relation: none, target: none) = {
  if type(source) != label {
    panic("edge source must be a label")
  }
  if type(relation) != label {
    panic("edge relation must be a label")
  }
  if type(target) != label {
    panic("edge target must be a label")
  }
  (source: source, relation: relation, target: target)
}

/// Place a semantic value in a source-anchored state slot.
///
/// This is the dynamic Typst representation of `State<T>`. `origin` identifies
/// the stable source owner of the slot; state evolution may replace `value`
/// while preserving that origin.
///
/// - value (any): The current semantic value.
/// - origin (content): The source-backed content anchoring this state slot.
/// -> dictionary
#let state(value, origin) = {
  if type(origin) != content {
    panic("state origin must be content")
  }
  (value: value, origin: origin)
}

/// Construct one node-local state from a node state and its outgoing edge
/// states.
///
/// Every outgoing edge must use the node state's ID as its source. Target
/// membership is a global graph concern and is intentionally not checked here.
///
/// - node (dictionary): A `State<Node>` value.
/// - outgoing (array): An array of `State<Edge>` values.
/// -> dictionary
#let local-state(node: none, outgoing: ()) = {
  if (
    type(node) != dictionary
      or "value" not in node
      or "origin" not in node
      or type(node.value) != dictionary
      or "id" not in node.value
      or "title" not in node.value
      or "metadata" not in node.value
      or type(node.value.id) != label
      or type(node.value.title) != content
      or type(node.value.metadata) != dictionary
      or type(node.origin) != content
  ) {
    panic("local node state must contain a State<Node>")
  }
  if type(outgoing) != array {
    panic("local node outgoing states must be an array")
  }

  for edge-state in outgoing {
    if (
      type(edge-state) != dictionary
        or "value" not in edge-state
        or "origin" not in edge-state
        or type(edge-state.value) != dictionary
        or "source" not in edge-state.value
        or "relation" not in edge-state.value
        or "target" not in edge-state.value
        or type(edge-state.value.source) != label
        or type(edge-state.value.relation) != label
        or type(edge-state.value.target) != label
        or type(edge-state.origin) != content
    ) {
      panic("local node outgoing states must contain State<Edge> values")
    }
    if edge-state.value.source != node.value.id {
      panic("outgoing edge source must equal the node state ID")
    }
  }

  (node: node, outgoing: outgoing)
}
