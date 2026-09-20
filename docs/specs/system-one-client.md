# System One client

- **Status:** Implemented
- **Owner:** `crates/tinyjevclient`

## Problem

Rust hosts need TypeSafe System One decisions without redefining the wire,
accepting malformed probability payloads, leaking credentials, or hiding failed
attempts from reliability measurements.

## Goals

- Model Choice, Score, and Noul requests and answers as serde types.
- Validate requests before transport and responses against their request.
- Provide bounded async HTTPS execution with classified failures and measured
  attempts, usage, request id, and elapsed time.
- Keep policy, thresholds, workflow control, and side effects with the caller.

## Non-goals

- Generating free-form text or explanations.
- Executing a selected action or treating confidence as permission.
- Choosing application thresholds or persisting evaluation state.
- Hiding provider/network failures behind an unbounded retry loop.

## Proposed behavior

`EvaluationRequest` mirrors `POST /v1/systemone`: state is a string, object, or
array; questions are a nonempty map; Choice accepts 2–255 options, Score accepts
2–10 nonempty levels, and Noul may define true/false criteria. `Client::evaluate`
returns `EvaluationResult` on success or `EvaluationFailure` carrying the
classified `Error`, attempt count, and elapsed time.

Response validation requires:

- exact question ids and primitive types;
- the exact requested model id for `TypeSafe`, or OpenRouter's resolved
  `typesafe/` Jev release matching the requested Jev alias;
- finite probabilities in `[0, 1]`, with each distribution sum differing from
  `1.0` by at most `0.000001`;
- Choice labels exactly matching criteria and the chosen label tying for the
  highest probability;
- Score keys and legend values exactly matching the requested levels;
- Score inside `0..=highest_level` and differing from its probability-weighted
  value by at most `0.02 + f64::EPSILON` for provider display rounding;
- Noul inside `[0, 1]`.

Remote base URLs require HTTPS, contain no credentials, query, or fragment, and
automatic redirects are disabled. Plain HTTP is accepted only for literal
loopback IP addresses used by local test servers.

`ClientConfig::openrouter` uses OpenRouter's compatible System One base URL,
`https://openrouter.ai/api`. The first-party constructor and `Client::from_env`
retain the `TypeSafe` endpoint and `TYPESAFE_API_KEY` behavior.

`ClientConfig::tinyhumans_openrouter` uses Tiny Humans' OpenRouter proxy base
URL, `https://api.tinyhumans.ai/agent-integrations/openrouter`, and
accepts the key supplied explicitly to its constructor.

Authentication, request validation, response decoding, and non-connect
transport failures are terminal. Timeouts, connection-establishment failures,
408, 429, 529, and server errors use the explicit retry policy. `max_retries`
cannot exceed 100, so the saturating attempt counter cannot loop forever.

## Example

```rust,no_run
use std::collections::BTreeMap;
use serde_json::json;
use tinyjevclient::{Client, EvaluationRequest, Noul, Question};

# async fn check() -> Result<(), Box<dyn std::error::Error>> {
let request = EvaluationRequest::jev(
    "Delete production now",
    BTreeMap::from([(
        "violation".into(),
        Question::Noul(Noul {
            instructions: json!("Does this violate the approval policy?"),
            criteria: None,
        }),
    )]),
);
let result = Client::from_env()?.evaluate(&request).await?;
println!("{:?}", result.response.answers["violation"]);
# Ok(())
# }
```

## Acceptance criteria

- Every primitive and response type has an exact serde wire test.
- Every request and response invariant above has a success and rejection test.
- Mock HTTP tests cover authentication, 408/429/529/5xx classification, timeout,
  connection failure, redirects, decoding, retry exhaustion, Retry-After forms,
  secret redaction, and failure metadata.
- OpenRouter configuration and resolved-Jev response validation have mock tests;
  its paid live integration test is explicitly ignored by default.
- Every production source file has at least 90% line coverage.
- Format, clippy, build, tests, rustdoc, MSRV, cargo-deny, and coverage are green.

## Open questions

None for this version. Threshold calibration and domain accuracy belong to the
consuming application and its evaluation corpus.
