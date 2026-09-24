#import "graph-node.typ": elements, observer
#import "metadata.typ": zk_metadata

#let emit = metadata

// A note declares its identity in its root heading, not its path or metadata.
#let zettel(metadata: zk_metadata, body) = {
  assert(
    type(metadata) == function,
    message: "zettel metadata must be a configured function",
  )
  let observe = observer(metadata: metadata())
  emit((
    protocol: "zettyp.note",
    version: 1,
    local: observe(body),
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
