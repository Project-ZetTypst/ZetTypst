#import "/core/lib.typ": eval, graph, policy

#let initial = (
  graph
    .assemble((
      graph.fragment(
        nodes: (graph.node("note", value: 2),),
      ),
    ))
    .state
)

#let left(state) = graph.assign(state, nodes: (
  note: state.values.nodes.note + 1,
))
#let right(state) = graph.assign(state, nodes: (
  note: state.values.nodes.note * 2,
))
#let merge(a, b) = graph.assign(a, nodes: (
  note: a.values.nodes.note - b.values.nodes.note,
))
#let tail(state) = graph.assign(state, nodes: (
  note: state.values.nodes.note * 10,
))

#let right-policy = policy.function("right", right)
#let calls(branch, join, end) = (
  (end.invoke)(inputs: ("merged",), output: "final"),
  (join.invoke)(inputs: ("left", "right"), output: "merged"),
  (branch.invoke)(inputs: ("initial",), output: "left"),
  (right-policy.invoke)(inputs: ("initial",), output: "right"),
)
#let run(declarations) = {
  let built = policy.assemble(
    declarations,
    inputs: ("initial",),
    output: "final",
  )
  assert.eq(built.issues, ())
  policy.evaluate(built.flow, topology: initial.graph, inputs: (initial,))
}

#let successful = calls(
  policy.function("left", left, check: state => ()),
  policy.function("merge", merge, inputs: 2),
  policy.function("tail", tail),
)

// These panics prove failed runs and blocked checks/runs are not called.
#let failing = calls(
  policy.function(
    "left",
    state => panic("failed policy ran"),
    check: state => ((kind: "rejected", message: "left is unavailable"),),
  ),
  policy.function(
    "merge",
    (a, b) => panic("blocked merge ran"),
    inputs: 2,
    check: (a, b) => panic("blocked merge checked"),
  ),
  policy.function(
    "tail",
    state => panic("blocked tail ran"),
    check: state => panic("blocked tail checked"),
  ),
)

#eval.announce(<policy.evaluation>, (
  initial: initial,
  direct: tail(merge(left(initial), right(initial))),
  success: run(successful),
  success-reordered: run(successful.rev()),
  failure: run(failing),
  failure-reordered: run(failing.rev()),
))
