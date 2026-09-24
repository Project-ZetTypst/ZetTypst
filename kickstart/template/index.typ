// Editable whole-project entrypoint. The manifest records paths, not IDs.
#import "@preview/zettyp-core:0.1.0": eval
#import ".zettypst/zk/source.typ": load

#let notes = load()

// Export observed identities and provenance without exporting full note bodies.
#eval.announce(<zk.notes>, notes.map(note => (
  id: str(note.state.value.id),
  title: note.state.value.title,
  metadata: note.state.value.metadata,
  path: note.path,
  origin: eval.inspect(note.state.origin),
)))

// Use the target heading's title for ordinary references.
#show ref: it => {
  if it.element != none and it.element.func() == heading {
    link(it.target)[[#it.element.body]]
  } else {
    it
  }
}

#for note in notes {
  note.body
}
