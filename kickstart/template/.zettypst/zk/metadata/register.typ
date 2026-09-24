// Schema helpers accept ordinary Typst predicates, so checks stay programmable.
#let of-type(expected) = value => type(value) == expected

#let array-of(check) = {
  assert(type(check) == function, message: "array-of expects a predicate")
  value => type(value) == array and value.all(check)
}

#let one-of(choices) = {
  assert(type(choices) == array, message: "one-of expects an array")
  value => value in choices
}

#let field(default, check: value => true) = {
  assert(type(check) == function, message: "field check must be a predicate")
  assert(check(default), message: "field default failed validation")
  (default: default, check: check)
}

// Realize a configured function once, after all .with(...) layers are applied.
// Fields are replaced, not deep-merged. Extend the schema to add new fields.
#let register(schema: (:), ..delta) = {
  assert(
    type(schema) == dictionary,
    message: "metadata schema must be a dictionary",
  )
  assert(
    delta.pos().len() == 0,
    message: "metadata delta must contain only named fields",
  )
  let changes = delta.named()
  for key in changes.keys() {
    assert(key in schema, message: "unknown metadata field: " + key)
  }

  let values = (:)
  for (key, spec) in schema {
    assert(type(spec) == dictionary, message: "invalid field schema: " + key)
    assert(
      "default" in spec and "check" in spec,
      message: "invalid field schema: " + key,
    )
    let check = spec.check
    assert(
      type(check) == function,
      message: "field check must be a predicate: " + key,
    )
    let value = changes.at(key, default: spec.default)
    assert(check(value), message: "invalid metadata field: " + key)
    values.insert(key, value)
  }
  values
}
