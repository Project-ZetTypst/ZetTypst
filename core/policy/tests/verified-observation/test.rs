use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};
use typst::foundations::Dict;
use zettyp_eval::Runtime;

fn report() -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut runtime = Runtime::new(root).unwrap();
    let evaluation = runtime
        .evaluate(
            "core/policy/tests/verified-observation/main.typ",
            Dict::new(),
        )
        .unwrap();
    let output = serde_json::to_value(evaluation.result.output.as_ref().unwrap()).unwrap();
    assert_eq!(output.as_object().unwrap().len(), 1);
    let announcements = output["policy.verified-observation"].as_array().unwrap();
    assert_eq!(announcements.len(), 1);
    announcements[0].clone()
}

#[test]
fn verified_early_observation_reaches_external_consumers() {
    let report = report();
    assert_eq!(report["domain-size"], 4);
    assert_eq!(
        report["verification"],
        json!({"has-plan": true, "issues": []})
    );
    let success = &report["success"];
    for name in ["early", "gated", "final"] {
        assert_eq!(success[name]["status"], "available");
        assert_eq!(success[name]["value"], true);
        assert_eq!(success[name]["target"], "final");
    }
    assert_eq!(success["early"]["product"], "initial");
    assert_eq!(success["gated"]["product"], "gated");
    assert_eq!(success["final"]["product"], "final");
    assert_eq!(success["early"]["evidence"]["kind"], "local-preservation");
    assert_eq!(
        success["early"]["evidence"]["edges"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(success["final"]["evidence"]["kind"], "final");
    let output = &success["execution"]["output"];
    assert_eq!(output["status"], "success");
    assert_eq!(
        success["early"]["value"],
        output["value"]["values"]["nodes"]["note"]["enabled"]
    );
    assert_eq!(
        output["value"]["values"]["nodes"]["note"]["phase"],
        json!({
            "type": "phase", "variant": "after", "value": "after",
        })
    );
}

#[test]
fn downstream_failure_preserves_only_available_binding_locations() {
    let report = report();
    let failure = &report["failure"];
    assert_eq!(failure["early"]["status"], "available");
    assert_eq!(failure["early"]["value"], false);
    assert_eq!(failure["early"]["product"], "initial");
    assert_eq!(
        failure["early"]["evidence"],
        report["success"]["early"]["evidence"]
    );
    assert_eq!(
        failure["gated"],
        json!({
            "status": "unavailable", "reason": "failure", "product": "gated", "target": "final",
        })
    );
    assert_eq!(
        failure["final"],
        json!({
            "status": "unavailable", "reason": "blocked", "product": "final", "target": "final",
        })
    );
    let results = &failure["execution"]["results"];
    assert_eq!(results["derived"]["status"], "success");
    assert_eq!(
        results["gated"],
        json!({"status": "failure", "issues": [{"kind": "disabled"}]})
    );
    assert_eq!(
        results["final"],
        json!({"status": "blocked", "dependencies": ["gated"]})
    );
}

#[test]
fn local_changes_are_rejected_even_when_later_calls_restore_the_value() {
    let report = report();
    assert_eq!(report["restored"], true);
    for (case, observation, invocations) in [
        ("invalid-phase", "phase", BTreeSet::from(["derived"])),
        (
            "invalid-compensation",
            "enabled",
            BTreeSet::from(["middle", "final"]),
        ),
    ] {
        assert_eq!(report[case]["has-plan"], false);
        let issues = report[case]["issues"].as_array().unwrap();
        assert!(!issues.is_empty());
        let actual: BTreeSet<_> = issues
            .iter()
            .map(|issue| {
                assert_eq!(issue["kind"], "observation-changed");
                assert_eq!(issue["observation"], observation);
                assert_eq!(issue["product"], "initial");
                assert_ne!(issue["before"], issue["after"]);
                assert_eq!(issue["inputs"].as_array().unwrap().len(), 1);
                assert_eq!(issue["port"], 0);
                issue["invocation"].as_str().unwrap()
            })
            .collect();
        assert_eq!(actual, invocations);
    }
}
