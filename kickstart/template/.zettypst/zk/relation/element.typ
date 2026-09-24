// Body-local dependency declarations. Colors are semantic relation labels.
#let colors = (
  replaces: <zk.replaces>,
  evolves-from: <zk.evolves-from>,
)

// Retain original content for reference discovery and source provenance.
// The marker is not a graph edge yet: its source is supplied by the enclosing
// zettel's observer. Rendering the body preserves native reference behavior.
#let colored-edge(color, body) = {
  assert(type(color) == label, message: "dependency color must be a label")
  assert(type(body) == content, message: "dependency body must be content")
  metadata((
    protocol: "zettyp.dependency",
    version: 1,
    color: color,
    body: body,
  ))
  body
}

#let replaces = colored-edge.with(colors.replaces)
#let evolves-from = colored-edge.with(colors.evolves-from)
