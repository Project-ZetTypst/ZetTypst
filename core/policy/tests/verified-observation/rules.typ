#import "/core/lib.typ": graph, policy, semantic, vocabulary

#let phase = vocabulary.register("phase", before: "before", after: "after")
#let initial = (
  graph
    .assemble((
      graph.fragment(
        nodes: (
          graph.node("note", value: (enabled: true, phase: phase.before)),
        ),
      ),
    ))
    .state
)
#let contract = semantic.contract(
  initial,
  registry: vocabulary.registry(phase: phase),
)

#let read-enabled(state) = state.values.nodes.note.enabled
#let read-phase(state) = state.values.nodes.note.phase
#let update(state, ..fields) = graph.assign(state, nodes: (
  note: state.values.nodes.note + fields.named(),
))

#let advance = policy.function("advance", state => update(
  state,
  phase: phase.after,
))
#let gate = policy.function(
  "gate",
  state => state,
  check: state => if read-enabled(state) { () } else {
    ((kind: "disabled"),)
  },
)
#let identity = policy.function("identity", state => state)
#let flip = policy.function("flip", state => update(
  state,
  enabled: not read-enabled(state),
))

#let call(definition, input, output) = (definition.invoke)(
  inputs: (input,),
  output: output,
)
#let calls = (
  call(identity, "gated", "final"),
  call(gate, "derived", "gated"),
  call(advance, "initial", "derived"),
)
