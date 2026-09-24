// Allocate an identity only when registering a new node. Persist the returned
// label in source; never recompute it when a note is renamed or moved.
// The caller supplies one request's time and identities observed from notes.
#let allocate(now, used) = {
  assert(type(now) == datetime, message: "ID allocation requires a datetime")
  assert(
    now.year() != none and now.hour() != none,
    message: "ID allocation requires both a date and a time",
  )
  assert(type(used) == array, message: "existing identities must be an array")
  assert(
    used.all(id => type(id) == label),
    message: "existing identities must be labels",
  )

  let base = now.display("[year][month][day]T[hour][minute][second]")
  let candidate = label(base)
  let suffix = 0
  while candidate in used {
    suffix += 1
    candidate = label(base + "-" + str(suffix))
  }
  candidate
}
