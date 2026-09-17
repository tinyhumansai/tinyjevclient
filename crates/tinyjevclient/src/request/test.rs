//! Request wire and validation tests.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use serde_json::json;

use super::*;

fn questions() -> BTreeMap<String, Question> {
    BTreeMap::from([
        (
            "route".to_owned(),
            Question::Choice(Choice {
                instructions: json!("Which agent should answer?"),
                criteria: BTreeMap::from([
                    ("planner".to_owned(), Some(json!("plans work"))),
                    ("reviewer".to_owned(), None),
                ]),
            }),
        ),
        (
            "quality".to_owned(),
            Question::Score(Score {
                instructions: json!("How strong is the evidence?"),
                criteria: vec![json!("unsupported"), json!("direct")],
            }),
        ),
        (
            "unsafe".to_owned(),
            Question::Noul(Noul {
                instructions: json!("Does this violate a stated constraint?"),
                criteria: Some(NoulCriteria {
                    r#true: json!("a constraint is violated"),
                    r#false: json!("all constraints are satisfied"),
                }),
            }),
        ),
    ])
}

#[test]
fn every_primitive_pins_its_wire_shape() {
    let request = EvaluationRequest::jev(json!({"message": "review this"}), questions());
    assert_eq!(
        serde_json::to_value(request).unwrap(),
        json!({
            "state": {"message": "review this"},
            "model": "jev-latest",
            "questions": {
                "quality": {
                    "type": "score",
                    "instructions": "How strong is the evidence?",
                    "criteria": ["unsupported", "direct"]
                },
                "route": {
                    "type": "choice",
                    "instructions": "Which agent should answer?",
                    "criteria": {"planner": "plans work", "reviewer": null}
                },
                "unsafe": {
                    "type": "noul",
                    "instructions": "Does this violate a stated constraint?",
                    "criteria": {
                        "true": "a constraint is violated",
                        "false": "all constraints are satisfied"
                    }
                }
            }
        })
    );
}

#[test]
fn accepts_string_object_and_array_state() {
    for state in [json!("text"), json!({"field": true}), json!([1, 2])] {
        EvaluationRequest::jev(state, questions())
            .validate()
            .unwrap();
    }
}

#[test]
fn rejects_empty_or_wrongly_shaped_inputs() {
    let cases = [
        EvaluationRequest::jev(json!(null), questions()),
        EvaluationRequest::jev(json!(true), questions()),
        EvaluationRequest::jev("state", BTreeMap::new()),
    ];
    for request in cases {
        assert!(request.validate().is_err());
    }
}

#[test]
fn enforces_choice_and_score_bounds() {
    let choice = EvaluationRequest::jev(
        "state",
        BTreeMap::from([(
            "choice".to_owned(),
            Question::Choice(Choice {
                instructions: json!("choose"),
                criteria: BTreeMap::from([("only".to_owned(), None)]),
            }),
        )]),
    );
    let score = EvaluationRequest::jev(
        "state",
        BTreeMap::from([(
            "score".to_owned(),
            Question::Score(Score {
                instructions: json!("score"),
                criteria: vec![json!("only")],
            }),
        )]),
    );
    assert!(choice.validate().is_err());
    assert!(score.validate().is_err());
}
