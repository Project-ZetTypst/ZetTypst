#import "zettel.typ": observations

// Importing this module does not read the manifest or load notes. Configuration
// extraction can therefore run independently of a generated source.toml.
#let load(manifest: "/.zettypst/source.toml") = {
  let paths = toml(manifest).paths
  assert(
    type(paths) == array,
    message: "source manifest paths must be an array",
  )
  assert(
    paths.all(path => type(path) == str),
    message: "source paths must be strings",
  )
  assert(paths.dedup().len() == paths.len(), message: "duplicate source path")

  let notes = ()
  let ids = ()
  for path in paths {
    assert(
      not path.contains("\\")
        and path.split("/").all(part => part not in ("", ".", "..")),
      message: "source path must be project-relative with forward slashes: "
        + path,
    )
    let records = observations(include ("/" + path))
    assert(
      records.len() > 0,
      message: "source has no zettel declaration: " + path,
    )
    for record in records {
      let id = record.local.node.value.id
      assert(id not in ids, message: "duplicate note ID: " + repr(id))
      ids.push(id)
      notes.push((local: record.local, body: record.body, path: path))
    }
  }
  notes
}
