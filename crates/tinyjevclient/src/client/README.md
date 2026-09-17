# Client module

The client validates a request, sends it to the System One endpoint, classifies
HTTP failures, retries only transient failures, validates the response against
the original questions, and returns attempts and end-to-end latency. API keys
remain private and render only as `[REDACTED]`.
