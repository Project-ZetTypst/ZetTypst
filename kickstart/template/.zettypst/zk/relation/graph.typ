// Dependency policy over a core GraphState. Original edge occurrences remain
// untouched; only this temporary view merges parallel declarations.
#import "@preview/zettyp-core:0.1.0": graph, node
#import "element.typ": colors

#import "validate.typ": dependency-colors, validate

// Strict document consumers fail explicitly; editor consumers use evaluate.
#let require-valid(result) = {
  if result.issues.len() > 0 {
    panic(result.issues.map(issue => issue.message).join("\n"))
  }
  result.value
}

#let dag(initial, dependencies: dependency-colors) = require-valid(
  validate(initial, dependencies: dependencies),
)

// Always derive from the original local prefixes, not a previously derived
// graph. This policy needs direct incoming edges, not recursive propagation.
#let evaluate(initial, dependencies: dependency-colors) = {
  let checked = validate(initial, dependencies: dependencies)
  if checked.value == none { return checked }
  let view = checked.value
  let nodes = initial
    .value
    .nodes
    .enumerate()
    .map(((index, value)) => {
      let incoming = view.edges.filter(group => group.value.target == value.id)
      let successors(color) = incoming
        .filter(group => group.value.relation == color)
        .map(group => group.value.source)
        .dedup()
      let replaced-by = successors(colors.replaces)
      let evolved-into = successors(colors.evolves-from)
      let metadata = value.metadata
      metadata.insert("replaced-by", replaced-by)
      metadata.insert("evolved-into", evolved-into)
      if replaced-by.len() > 0 {
        metadata.insert("relation", "archived")
      } else if evolved-into.len() > 0 {
        metadata.insert("relation", "legacy")
      }
      node.state(
        node.node(id: value.id, title: value.title, metadata: metadata),
        initial.origin.nodes.at(index),
      )
    })
  let final = graph.state(
    nodes: nodes,
    edges: initial
      .value
      .edges
      .enumerate()
      .map(((index, value)) => (
        node.state(value, initial.origin.edges.at(index))
      )),
  )
  (value: final, issues: ())
}

#let derive(initial, dependencies: dependency-colors) = require-valid(
  evaluate(initial, dependencies: dependencies),
)
