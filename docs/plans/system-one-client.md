# Implement the System One client

Linked specification: [`../specs/system-one-client.md`](../specs/system-one-client.md).

1. Replace the TinyBus template with one ordinary Rust library crate.
2. Define and pin Choice, Score, Noul, request, answer, usage, and response wires.
3. Validate requests and request-relative response invariants.
4. Add a rustls client with secret redaction, classified failures, and bounded retries.
5. Test all wire, validation, transport, retry, and public API behavior.
