//! Response wire and request-relative validation tests.

#![allow(clippy::unwrap_used, clippy::panic)]

use std::collections::BTreeMap;

use serde_json::json;

use super::*;
use crate::{Choice, EvaluationRequest, Noul, Question, Score};

fn request() -> EvaluationRequest {
    EvaluationRequest::jev(
        "state",
        BTreeMap::from([
            (
                "route".to_owned(),
                Question::Choice(Choice {
                    instructions: json!("route"),
                    criteria: BTreeMap::from([("a".to_owned(), None), ("b".to_owned(), None)]),
                }),
            ),
            (
                "quality".to_owned(),
                Question::Score(Score {
                    instructions: json!("quality"),
                    criteria: vec![json!("low"), json!("high")],
                }),
            ),
            (
                "safe".to_owned(),
                Question::Noul(Noul {
                    instructions: json!("safe"),
                    criteria: None,
                }),
            ),
        ]),
    )
}

fn response() -> EvaluationResponse {
    serde_json::from_value(json!({
        "model": "jev-latest",
        "answers": {
            "route": {
                "type": "choice",
                "choice": "b",
                "probabilities": {"a": 0.25, "b": 0.75},
                "confidence": 0.5
            },
            "quality": {
                "type": "score",
                "score": 0.8,
                "legend": {"0": "low", "1": "high"},
                "probabilities": {"0": 0.2, "1": 0.8},
                "confidence": 0.6
            },
            "safe": {"type": "noul", "noul": 0.9}
        },
        "usage": {"input_tokens": 42, "output_tokens": 3}
    }))
    .unwrap()
}

#[test]
fn validates_all_three_answer_types() {
    response().validate_for(&request()).unwrap();
}

#[test]
fn usage_fields_remain_optional() {
    let usage: Usage = serde_json::from_value(json!({})).unwrap();
    assert_eq!(usage, Usage::default());
}

#[test]
fn rejects_missing_extra_or_wrongly_typed_answers() {
    let mut missing = response();
    missing.answers.remove("safe");
    assert!(missing.validate_for(&request()).is_err());

    let mut wrong = response();
    wrong.answers.insert(
        "safe".to_owned(),
        Answer::Choice(ChoiceAnswer {
            choice: "a".to_owned(),
            probabilities: BTreeMap::from([("a".to_owned(), 1.0)]),
            confidence: 1.0,
        }),
    );
    assert!(wrong.validate_for(&request()).is_err());
}

#[test]
fn rejects_invalid_distributions_and_inconsistent_scores() {
    let mut distribution = response();
    let Answer::Choice(choice) = distribution.answers.get_mut("route").unwrap() else {
        panic!("fixture answer should be a choice")
    };
    choice.probabilities.insert("a".to_owned(), 0.75);
    assert!(distribution.validate_for(&request()).is_err());

    let mut score = response();
    let Answer::Score(answer) = score.answers.get_mut("quality").unwrap() else {
        panic!("fixture answer should be a score")
    };
    answer.score = 0.1;
    assert!(score.validate_for(&request()).is_err());
}
