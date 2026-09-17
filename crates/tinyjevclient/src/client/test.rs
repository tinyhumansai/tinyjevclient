//! Client transport, retry, measurement, and secret-handling tests.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

use super::*;
use crate::{Choice, Question};

fn request() -> EvaluationRequest {
    EvaluationRequest::jev(
        json!({"message": "review this"}),
        BTreeMap::from([(
            "route".to_owned(),
            Question::Choice(Choice {
                instructions: json!("Who should answer?"),
                criteria: BTreeMap::from([("alice".to_owned(), None), ("bob".to_owned(), None)]),
            }),
        )]),
    )
}

fn success() -> String {
    json!({
        "model": "jev-latest",
        "answers": {
            "route": {
                "type": "choice",
                "choice": "bob",
                "probabilities": {"alice": 0.2, "bob": 0.8},
                "confidence": 0.6
            }
        },
        "usage": {"input_tokens": 12, "output_tokens": 2}
    })
    .to_string()
}

fn response(status: u16, body: &str, extra_headers: &str) -> String {
    let reason = if status == 200 { "OK" } else { "Error" };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra_headers}\r\n{body}",
        body.len()
    )
}

async fn server(responses: Vec<String>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&requests);
    tokio::spawn(async move {
        for reply in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0_u8; 16_384];
            let count = stream.read(&mut buffer).await.unwrap();
            recorded
                .lock()
                .await
                .push(String::from_utf8_lossy(&buffer[..count]).into_owned());
            stream.write_all(reply.as_bytes()).await.unwrap();
        }
    });
    (format!("http://{address}"), requests)
}

fn config(base_url: String) -> ClientConfig {
    let mut config = ClientConfig::new("secret-test-key");
    config.base_url = base_url;
    config.timeout = Duration::from_secs(1);
    config.retry = RetryPolicy {
        max_retries: 0,
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(5),
    };
    config
}

#[tokio::test]
async fn sends_the_documented_endpoint_and_bearer_header() {
    let (base_url, requests) = server(vec![response(
        200,
        &success(),
        "x-request-id: request-7\r\n",
    )])
    .await;
    let result = Client::new(config(base_url))
        .unwrap()
        .evaluate(&request())
        .await
        .unwrap();
    assert_eq!(result.attempts, 1);
    assert_eq!(result.request_id.as_deref(), Some("request-7"));
    assert_eq!(result.response.usage.input_tokens, Some(12));
    let sent = requests.lock().await.join("");
    assert!(sent.starts_with("POST /v1/systemone HTTP/1.1"));
    assert!(
        sent.to_ascii_lowercase()
            .contains("authorization: bearer secret-test-key")
    );
    assert!(sent.contains("\"model\":\"jev-latest\""));
}

#[tokio::test]
async fn retries_rate_limits_and_reports_attempts() {
    let (base_url, requests) =
        server(vec![response(429, "{}", ""), response(200, &success(), "")]).await;
    let mut config = config(base_url);
    config.retry.max_retries = 1;
    let result = Client::new(config)
        .unwrap()
        .evaluate(&request())
        .await
        .unwrap();
    assert_eq!(result.attempts, 2);
    assert_eq!(requests.lock().await.len(), 2);
}

#[tokio::test]
async fn authentication_is_terminal() {
    let (base_url, requests) = server(vec![response(401, "{}", "")]).await;
    let error = Client::new(config(base_url))
        .unwrap()
        .evaluate(&request())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Authentication));
    assert_eq!(requests.lock().await.len(), 1);
}

#[tokio::test]
async fn malformed_success_body_is_a_decode_failure() {
    let (base_url, _) = server(vec![response(200, "not-json", "")]).await;
    let error = Client::new(config(base_url))
        .unwrap()
        .evaluate(&request())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Decode { .. }));
}

#[tokio::test]
async fn debug_output_redacts_the_api_key() {
    let rendered = format!("{:?}", config("http://127.0.0.1:1".to_owned()));
    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("secret-test-key"));
}

#[test]
fn rejects_invalid_configuration_before_transport() {
    let mut empty = ClientConfig::new("");
    empty.base_url = "not a URL".to_owned();
    assert!(matches!(
        Client::new(empty),
        Err(Error::InvalidConfig { .. })
    ));
}
