#import "/core/lib.typ": eval, policy

#let relation = policy.function("relation", x => panic("must not execute"))
#let context-policy = policy.function("context", x => panic("must not execute"))
#let merge = policy.function(
  "merge",
  (x, y) => panic("must not execute"),
  inputs: 2,
)

#let result = policy.assemble(
  (
    (merge.invoke)(inputs: ("related", "contextual"), output: "semantic"),
    (relation.invoke)(inputs: ("initial",), output: "related"),
    (context-policy.invoke)(inputs: ("initial",), output: "contextual"),
  ),
  inputs: ("initial",),
  output: "semantic",
)

// Recover argument order from the assembled edges.
#let observe(flow) = {
  let state = flow.state
  (
    inputs: flow.inputs,
    output: flow.output,
    layers: flow.layers,
    nodes: state.graph.nodes.map(id => (
      id: id,
      kind: state.values.nodes.at(id).kind,
      arguments: state
        .graph
        .edges
        .pairs()
        .filter(pair => pair.at(1).target == id)
        .sorted(key: pair => state.values.edges.at(pair.at(0)).port)
        .map(pair => pair.at(1).source),
    )),
  )
}

#assert.eq(result.issues, ())
#eval.announce(<policy.test>, observe(result.flow))
