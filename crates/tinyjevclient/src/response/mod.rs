//! Typed System One responses and cross-checks against their requests.

#[cfg(test)]
mod test;

mod types;

pub use types::{Answer, ChoiceAnswer, EvaluationResponse, NoulAnswer, ScoreAnswer, Usage};

use std::collections::BTreeSet;

use crate::{Error, EvaluationRequest, Question, Result};

const PROBABILITY_TOLERANCE: f64 = 0.000_001;
const SCORE_TOLERANCE: f64 = 0.02;

impl EvaluationResponse {
    /// Check this response against the request that produced it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidResponse`] when answer ids or primitive types do
    /// not match the request, or when a probability payload is inconsistent.
    pub fn validate_for(&self, request: &EvaluationRequest) -> Result<()> {
        self.validate_for_model(request, |response_model| response_model == request.model)
    }

    /// Check an `OpenRouter` System One response against its request.
    ///
    /// `OpenRouter` resolves bare Jev model IDs into the `typesafe/` namespace,
    /// so a response can name a concrete release when the request used an
    /// alias such as `jev-latest`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidResponse`] when answer ids or primitive types do
    /// not match the request, or when a probability payload is inconsistent.
    pub fn validate_for_openrouter(&self, request: &EvaluationRequest) -> Result<()> {
        let requested = request.model.trim_start_matches('~');
        let expected = if requested.contains('/') {
            requested.to_owned()
        } else {
            format!("typesafe/{requested}")
        };
        self.validate_for_model(request, |response_model| {
            if requested == "jev-latest" {
                response_model.starts_with("typesafe/jev-")
            } else {
                response_model == expected || response_model.starts_with(&format!("{expected}-"))
            }
        })
    }

    fn validate_for_model(
        &self,
        request: &EvaluationRequest,
        model_matches: impl FnOnce(&str) -> bool,
    ) -> Result<()> {
        if self.model.trim().is_empty() {
            return Err(Error::invalid_response("response model must not be empty"));
        }
        if !model_matches(&self.model) {
            return Err(Error::invalid_response(
                "response model must match the requested model",
            ));
        }
        let expected: BTreeSet<&str> = request.questions.keys().map(String::as_str).collect();
        let actual: BTreeSet<&str> = self.answers.keys().map(String::as_str).collect();
        if actual != expected {
            return Err(Error::invalid_response(
                "response answer ids must exactly match request question ids",
            ));
        }
        for (id, question) in &request.questions {
            let answer = self
                .answers
                .get(id)
                .ok_or_else(|| Error::invalid_response("response answer is missing"))?;
            validate_pair(question, answer)?;
        }
        Ok(())
    }
}

fn validate_pair(question: &Question, answer: &Answer) -> Result<()> {
    match (question, answer) {
        (Question::Choice(question), Answer::Choice(answer)) => {
            validate_probability(answer.confidence, "choice confidence")?;
            validate_distribution(&answer.probabilities, "choice")?;
            let expected: BTreeSet<&str> = question.criteria.keys().map(String::as_str).collect();
            let actual: BTreeSet<&str> = answer.probabilities.keys().map(String::as_str).collect();
            if actual != expected || !expected.contains(answer.choice.as_str()) {
                return Err(Error::invalid_response(
                    "choice labels must exactly match request criteria",
                ));
            }
            let selected = answer.probabilities[&answer.choice];
            if answer
                .probabilities
                .values()
                .any(|probability| *probability > selected + PROBABILITY_TOLERANCE)
            {
                return Err(Error::invalid_response(
                    "choice must name a highest-probability option",
                ));
            }
        }
        (Question::Score(question), Answer::Score(answer)) => {
            validate_probability(answer.confidence, "score confidence")?;
            validate_distribution(&answer.probabilities, "score")?;
            let expected: BTreeSet<String> = (0..question.criteria.len())
                .map(|index| index.to_string())
                .collect();
            let actual: BTreeSet<String> = answer.probabilities.keys().cloned().collect();
            let legend: BTreeSet<String> = answer.legend.keys().cloned().collect();
            let legend_matches = question
                .criteria
                .iter()
                .enumerate()
                .all(|(index, criterion)| answer.legend.get(&index.to_string()) == Some(criterion));
            if actual != expected || legend != expected || !legend_matches {
                return Err(Error::invalid_response(
                    "score levels must exactly match request criteria",
                ));
            }
            let maximum =
                u32::try_from(question.criteria.len() - 1).map_or(f64::from(u32::MAX), f64::from);
            if !answer.score.is_finite() || !(0.0..=maximum).contains(&answer.score) {
                return Err(Error::invalid_response(
                    "score must be finite and inside the requested scale",
                ));
            }
            let expected_score: f64 = answer
                .probabilities
                .iter()
                .map(|(level, probability)| level.parse::<f64>().unwrap_or_default() * probability)
                .sum();
            if (answer.score - expected_score).abs() > SCORE_TOLERANCE + f64::EPSILON {
                return Err(Error::invalid_response(
                    "score must equal the probability-weighted level",
                ));
            }
        }
        (Question::Noul(_), Answer::Noul(answer)) => {
            validate_probability(answer.noul, "noul")?;
        }
        _ => {
            return Err(Error::invalid_response(
                "answer type must match its request question type",
            ));
        }
    }
    Ok(())
}

fn validate_distribution(
    probabilities: &std::collections::BTreeMap<String, f64>,
    name: &str,
) -> Result<()> {
    if probabilities.is_empty() {
        return Err(Error::invalid_response(format!(
            "{name} probabilities must not be empty"
        )));
    }
    for probability in probabilities.values() {
        validate_probability(*probability, name)?;
    }
    let sum: f64 = probabilities.values().sum();
    if (sum - 1.0).abs() > PROBABILITY_TOLERANCE {
        return Err(Error::invalid_response(format!(
            "{name} probabilities must sum to one"
        )));
    }
    Ok(())
}

fn validate_probability(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(Error::invalid_response(format!(
            "{name} must be between zero and one"
        )));
    }
    Ok(())
}
