#import "@preview/zettyp-core:0.1.0": node
#import "metadata.typ": zk_metadata

#let emit = metadata

// Inspect ordinary sequences and style wrappers, without entering nested
// containers or deferred context. Keep the original content for presentation.
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

// A note declares its identity in its root heading, not its path or metadata.
#let zettel(metadata: zk_metadata, body) = {
  assert(
    type(metadata) == function,
    message: "zettel metadata must be a configured function",
  )
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
  let initial = metadata()
  emit((
    protocol: "zettyp.note",
    version: 1,
    state: node.state(
      node.node(id: id, title: root.body, metadata: initial),
      root,
    ),
    body: body,
  ))
}

// A file can yield multiple explicit zettel declarations. Section extraction
// within one zettel is a separate policy, not part of this initial format.
#let observations(body) = (
  elements(body)
    .filter(it => {
      if it.func() != emit { return false }
      let value = it.value
      (
        type(value) == dictionary
          and value.at("protocol", default: none) == "zettyp.note"
          and value.at("version", default: none) == 1
      )
    })
    .map(it => it.value)
)
