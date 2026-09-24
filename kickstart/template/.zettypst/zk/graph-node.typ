// Node-local observation. Identity is declared by the
// zettel's root heading; it is not restricted to a generated ID format.
#import "@preview/zettyp-core:0.1.0": node

#let relations = (ref: <zk.ref>)

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

// Keep each native ref occurrence, including targets not yet known to be nodes.
// Walk evaluated content, not source text. Do not descend into
// ref supplements or metadata payloads: they may repeat the original content.
#let references(value) = {
  if type(value) == array {
    value.fold((), (found, child) => found + references(child))
  } else if type(value) == content {
    let fields = value.fields()
    if value.func() == ref {
      (value,)
    } else if value.func() == metadata {
      ()
    } else if "children" in fields {
      references(fields.children)
    } else if "child" in fields {
      references(fields.child)
    } else if "body" in fields {
      references(fields.body)
    } else {
      ()
    }
  } else {
    ()
  }
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
      outgoing: references(body).map(reference => node.state(
        node.edge(
          source: id,
          relation: relations.ref,
          target: reference.target,
        ),
        reference,
      )),
    )
  }
}
