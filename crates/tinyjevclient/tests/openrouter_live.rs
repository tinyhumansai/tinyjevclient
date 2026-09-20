//! Explicit paid integration coverage for `OpenRouter`'s System One endpoint.

use std::collections::BTreeMap;

use serde_json::json;
use tinyjevclient::{Client, EvaluationRequest, Noul, Question};

/// Verifies that JEV evaluates a typed question through `OpenRouter`.
///
/// This test is ignored because it spends a paid API call and requires
/// `OPENROUTER_API_KEY`.
#[tokio::test]
#[ignore = "requires OPENROUTER_API_KEY and makes a paid OpenRouter request"]
async fn jev_evaluates_through_openrouter() -> Result<(), Box<dyn std::error::Error>> {
    let request = EvaluationRequest::jev(
        json!({"ticket": "I was charged twice and want a refund."}),
        BTreeMap::from([(
            "refund".to_owned(),
            Question::Noul(Noul {
                instructions: json!("Is the customer asking for money back?"),
                criteria: None,
            }),
        )]),
    );

    let result = Client::from_openrouter_env()?.evaluate(&request).await?;
    assert!(result.response.model.starts_with("typesafe/jev-"));
    assert!(result.response.answers.contains_key("refund"));
    Ok(())
}
