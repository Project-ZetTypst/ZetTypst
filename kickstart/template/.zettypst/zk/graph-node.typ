// Node-local observation. Identity is declared by the
// zettel's root heading; it is not restricted to a generated ID format.
#import "@preview/zettyp-core:0.1.0": node

#import "relation.typ": outgoing, references, relations

// Flatten only ordinary sequences and style wrappers for declaration discovery.
#let elements(body) = {
  let fields = body.fields()
  if "styles" in fields and "child" in fields {
    elements(fields.child)
  } else if "children" in fields {
    fields.children.fold((), (all, child) => all + elements(child))
  } else {
    (body,)
  }
}

#let root-heading(it) = {
  if it.func() != heading { return false }
  let fields = it.fields()
  let level = fields.at("level", default: auto)
  if level == auto { level = fields.at("depth", default: none) }
  level == 1
}

// content -> LocalNodeState. Target membership is resolved during aggregation;
// local outgoing states retain all observed refs and their original content.
#let observer(metadata: (:)) = {
  assert(
    type(metadata) == dictionary,
    message: "note metadata must be a dictionary",
  )
  body => {
    assert(type(body) == content, message: "note body must be content")
    let roots = elements(body).filter(root-heading)
    assert(
      roots.len() == 1,
      message: "zettel requires exactly one level-one root heading",
    )
    let root = roots.first()
    let id = root.fields().at("label", default: none)
    assert(
      type(id) == label,
      message: "zettel root heading requires a persistent label",
    )
    node.local-state(
      node: node.state(
        node.node(id: id, title: root.body, metadata: metadata),
        root,
      ),
      outgoing: outgoing(body, id),
    )
  }
}
