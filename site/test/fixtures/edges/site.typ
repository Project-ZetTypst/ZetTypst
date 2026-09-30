// Two notes linking each other and themselves, laid out by the site card with
// every view repeating the body, so copied links are rendered too.
#import ".zettypst/lib.typ": evaluate, export-html, load
#import "card.typ": card

#let project = load()
#assert.eq(project.issues, ())
#export-html(project, evaluate(project), card: card.with(views: (
  expand: note => note.main,
)))
