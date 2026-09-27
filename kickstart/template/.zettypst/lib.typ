#import "@preview/zettyp-core:0.1.0": knowledge, vocabulary

#let lifecycle = vocabulary.register(
  "lifecycle",
  active: "active",
  legacy: "legacy",
  archived: "archived",
)
#let colors = vocabulary.register(
  "relation",
  ref: <zk.ref>,
  replaces: <zk.replaces>,
  evolves-from: <zk.evolves-from>,
)
#let emit = metadata

#let zk_metadata(
  aliases: (),
  abstract: "",
  tags: (),
  relation: lifecycle.active,
) = {
  assert(type(aliases) == array and aliases.all(it => type(it) == str))
  assert(type(abstract) == str and type(tags) == array)
  assert(relation in lifecycle.values(), message: "unknown lifecycle state")
  (
    value: (relation: relation),
    data: (aliases: aliases, abstract: abstract, tags: tags),
  )
}

// Shallow: declarations through sequences/styles. Deep: rendered bodies too.
// References and metadata are leaves: never walk supplements or payloads.
#let elements(body, deep: false) = {
  if type(body) == array {
    return body.map(it => elements(it, deep: deep)).flatten()
  }
  if type(body) != content { return () }
  if body.func() in (ref, emit) { return (body,) }
  let fields = body.fields()
  let keys = if deep { ("children", "child", "body") } else if (
    "styles" in fields
  ) {
    ("children", "child")
  } else { ("children",) }
  let key = keys.find(key => key in fields)
  if key == none { (body,) } else { elements(fields.at(key), deep: deep) }
}

#let declarations(body, protocol, deep: false) = (
  elements(body, deep: deep)
    .filter(it => (
      it.func() == emit
        and type(it.value) == dictionary
        and it.value.at("protocol", default: none) == protocol
        and it.value.at("version", default: none) == 1
    ))
    .map(it => it.value)
)
#let references(body) = elements(body, deep: true).filter(it => (
  it.func() == ref
))

#let colored-edge(color, body) = {
  assert(color in (colors.replaces, colors.evolves-from))
  emit((protocol: "zettyp.dependency", version: 1, color: color, body: body))
  body
}
#let replaces = colored-edge.with(colors.replaces)
#let evolves-from = colored-edge.with(colors.evolves-from)

#let read-note(body, metadata: zk_metadata()) = {
  assert(type(body) == content, message: "note body must be content")
  let roots = elements(body).filter(it => {
    if it.func() != heading { return false }
    let fields = it.fields()
    let level = fields.at("level", default: auto)
    if level == auto { level = fields.at("depth", default: none) }
    level == 1
  })
  assert.eq(
    roots.len(),
    1,
    message: "note requires exactly one level-one root heading",
  )
  let root = roots.first()
  let identity = root.fields().at("label", default: none)
  assert(
    type(identity) == label,
    message: "note root heading requires a persistent label",
  )
  let id = str(identity)
  let outgoing = (
    references(body).map(origin => (origin: origin, color: colors.ref))
      + declarations(body, "zettyp.dependency", deep: true)
        .map(marker => (
          references(marker.body).map(origin => (
            origin: origin,
            color: marker.color,
          ))
        ))
        .flatten()
  )
  knowledge.local(
    id,
    value: metadata.value,
    data: metadata.data + (title: root.body),
    origin: root,
    references: outgoing
      .enumerate()
      .map(((index, item)) => knowledge.reference(
        str(id.len()) + ":" + id + "/ref/" + str(index),
        target: str(item.origin.target),
        value: (relation: item.color),
        origin: item.origin,
      )),
  )
}
#let note = knowledge.raw-to-local(read-note)

#let zettel(metadata: zk_metadata, body) = {
  assert(
    type(metadata) == function,
    message: "zettel metadata must be a configured function",
  )
  emit((
    protocol: "zettyp.note",
    version: 1,
    local: (note.observe)(body, metadata: metadata()),
    body: body,
  ))
}
#let observations(body) = declarations(body, "zettyp.note")

// Loading is explicit: importing this configuration never reads the manifest.
#let load(manifest: "/.zettypst/source.toml") = {
  let paths = toml(manifest).at("paths", default: none)
  assert(
    type(paths) == array and paths.all(path => type(path) == str),
    message: "source manifest requires an array of paths",
  )
  assert.eq(paths.dedup().len(), paths.len(), message: "duplicate source path")
  assert(
    paths.all(path => (
      not path.contains("\\")
        and path.split("/").all(part => part not in ("", ".", ".."))
    )),
    message: "source paths must be project-relative with forward slashes",
  )
  let notes = paths
    .map(path => {
      let records = observations(include ("/" + path))
      assert(
        records.len() > 0,
        message: "source has no zettel declaration: " + path,
      )
      records.map(record => record + (path: path))
    })
    .flatten()
  knowledge.assemble(notes.map(note => note.local)) + (notes: notes)
}
