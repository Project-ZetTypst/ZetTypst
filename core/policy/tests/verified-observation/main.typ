#import "rules.typ": *
#import "/core/lib.typ": eval, observation, policy, semantic

#let build(calls) = {
  let result = policy.assemble(calls, inputs: ("initial",), output: "final")
  assert.eq(result.issues, ())
  result.flow
}
#let verify(flow, bindings) = {
  let prepared = observation.prepare(flow, bindings)
  assert.eq(prepared.issues, ())
  observation.verify(prepared.plan, contract: contract)
}

#let flow = build(calls)
#let verified = verify(flow, (
  observation.bind("early", observer: read-enabled, at: "initial"),
  observation.bind("gated", observer: read-enabled, at: "gated"),
  observation.bind(
    "final",
    observer: state => {
      assert(read-enabled(state), message: "blocked observer must not run")
      read-enabled(state)
    },
    at: "final",
  ),
))
#assert.eq(verified.issues, ())
#assert(verified.plan != none)

#let run(input) = {
  let execution = policy.evaluate(flow, contract: contract, inputs: (input,))
  let table = observation.collect(verified.plan, execution)
  (
    execution: execution,
    early: observation.query(table, "early"),
    gated: observation.query(table, "gated"),
    final: observation.query(table, "final"),
  )
}

#let invalid-phase = verify(flow, (
  observation.bind("phase", observer: read-phase, at: "initial"),
))
#let compensated = build((
  call(flip, "initial", "middle"),
  call(flip, "middle", "final"),
))
#let invalid-compensation = verify(compensated, (
  observation.bind("enabled", observer: read-enabled, at: "initial"),
))
#let summary(result) = (has-plan: result.plan != none, issues: result.issues)

#eval.announce(<policy.verified-observation>, (
  domain-size: semantic.states(contract).len(),
  verification: summary(verified),
  success: run(initial),
  failure: run(update(initial, enabled: false)),
  invalid-phase: summary(invalid-phase),
  invalid-compensation: summary(invalid-compensation),
  restored: policy
    .evaluate(compensated, contract: contract, inputs: (initial,))
    .output
    .value
    == initial,
))
