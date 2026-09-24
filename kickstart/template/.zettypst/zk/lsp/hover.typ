// Present the target node's current semantic value as plain-text hover content.
#import "@preview/zettyp-lsp:0.1.0" as lsp

#let display-value(value) = {
  if type(value) == content {
    let fields = value.fields()
    if repr(value.func()) == "space" {
      " "
    } else if value.func() in (linebreak, parbreak) {
      "\n"
    } else if "text" in fields {
      fields.text
    } else if "children" in fields {
      fields.children.map(display-value).join()
    } else if "child" in fields {
      display-value(fields.child)
    } else if "body" in fields {
      display-value(fields.body)
    } else {
      repr(value)
    }
  } else if type(value) == array {
    value.map(display-value).join(", ")
  } else if type(value) in (str, label) {
    str(value)
  } else {
    repr(value)
  }
}

#let hover-contents(node) = (
  kind: "plaintext",
  value: display-value(node.title)
    + "\n@" + str(node.id) + "\n\n"
    + node.metadata.keys().sorted().map(key => (
      key + ": " + display-value(node.metadata.at(key))
    )).join("\n"),
)

#let announce(targets) = {
  for target in targets {
    lsp.announce(lsp.effect-kinds.hover, lsp.hover(
      applies-to: target.origin,
      contents: hover-contents(target.node),
    ))
  }
}
