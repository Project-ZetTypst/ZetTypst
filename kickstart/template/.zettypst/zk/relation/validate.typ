// Semantic validation returns source-backed issues without depending on LSP.
#import "element.typ": colors

#let dependency-colors = (colors.replaces, colors.evolves-from)

#let reachable(edges, start, target) = {
  let pending = (start,)
  let visited = ()
  while pending.len() > 0 {
    let current = pending.pop()
    if current == target { return true }
    if current in visited { continue }
    visited.push(current)
    for group in edges {
      if group.value.source == current and group.value.target not in visited {
        pending.push(group.value.target)
      }
    }
  }
  false
}

#let validate(initial, dependencies: dependency-colors) = {
  let edges = ()
  for (index, edge) in initial.value.edges.enumerate() {
    if edge.relation not in dependencies { continue }
    let existing = edges.position(group => (
      group.value.source == edge.source and group.value.target == edge.target
    ))
    if existing == none {
      edges.push((value: edge, occurrences: (index,)))
    } else {
      let group = edges.at(existing)
      group.occurrences.push(index)
      edges.at(existing) = group
    }
  }

  let issues = ()
  let issue(code, message, group) = (
    code: code,
    message: message,
    occurrences: group.occurrences,
    origins: group.occurrences.map(index => initial.origin.edges.at(index)),
  )
  for group in edges {
    let edge = group.value
    let endpoints = repr(edge.source) + " -> " + repr(edge.target)
    let colors = group
      .occurrences
      .map(index => initial.value.edges.at(index).relation)
      .dedup()
    if colors.len() > 1 {
      issues.push(issue(
        <zk.relation.color-conflict>,
        "conflicting dependency colors: " + endpoints,
        group,
      ))
    }
    // An edge lies on a cycle exactly when its target can reach its source.
    // This excludes acyclic edges entering or leaving a cyclic component.
    if reachable(edges, edge.target, edge.source) {
      issues.push(issue(
        <zk.relation.cycle>,
        "dependency graph contains a cycle through " + endpoints,
        group,
      ))
    }
  }
  if issues.len() > 0 { return (value: none, issues: issues) }

  let ids = initial.value.nodes.map(it => it.id)
  let remaining = ids
  let pending = edges
  let order = ()
  while remaining.len() > 0 {
    let roots = remaining.filter(id => (
      not pending.any(group => group.value.target == id)
    ))
    order += roots
    remaining = remaining.filter(id => id not in roots)
    pending = pending.filter(group => group.value.source not in roots)
  }
  (value: (nodes: ids, edges: edges, order: order), issues: ())
}
