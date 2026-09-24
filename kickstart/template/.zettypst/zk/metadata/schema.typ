#import "register.typ": array-of, field, of-type, one-of

// Edit or extend these declarations to define the project's metadata prefix.
// Identity comes from note structure; graph-derived fields are computed later.
#let schema = (
  aliases: field((), check: array-of(of-type(str))),
  abstract: field("", check: of-type(str)),
  // Tag elements will be defined by the shared-vocabulary policy.
  tags: field((), check: of-type(array)),
  relation: field("active", check: one-of(("active", "legacy", "archived"))),
)
