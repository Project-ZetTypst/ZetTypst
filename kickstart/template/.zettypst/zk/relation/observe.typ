#import "@preview/zettyp-core:0.1.0": node

#let relations = (ref: <zk.ref>)

// Walk rendered content, never ref supplements or arbitrary metadata payloads.
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

// Read each marker once. Nested markers are discovered in the rendered body;
// each enclosing marker independently colors all refs in its own body.
#let dependencies(value, source) = {
  if type(value) == array {
    value.fold((), (found, child) => found + dependencies(child, source))
  } else if type(value) == content {
    let fields = value.fields()
    if value.func() == metadata {
      let declaration = fields.value
      if (
        type(declaration) == dictionary
          and declaration.at("protocol", default: none) == "zettyp.dependency"
          and declaration.at("version", default: none) == 1
      ) {
        references(declaration.body).map(reference => node.state(
          node.edge(
            source: source,
            relation: declaration.color,
            target: reference.target,
          ),
          reference,
        ))
      } else {
        ()
      }
    } else if value.func() == ref {
      ()
    } else if "children" in fields {
      dependencies(fields.children, source)
    } else if "child" in fields {
      dependencies(fields.child, source)
    } else if "body" in fields {
      dependencies(fields.body, source)
    } else {
      ()
    }
  } else {
    ()
  }
}

#let outgoing(body, source) = {
  (
    references(body).map(reference => node.state(
      node.edge(
        source: source,
        relation: relations.ref,
        target: reference.target,
      ),
      reference,
    ))
      + dependencies(body, source)
  )
}
