# Implement the System One client

Linked specification: [`../specs/system-one-client.md`](../specs/system-one-client.md).

1. Replace the TinyBus template with `crates/tinyjevclient/Cargo.toml` and
   `crates/tinyjevclient/src/lib.rs`; remove the obsolete module/contract crates,
   submodule, packaging workflow, and release documentation. Compile the empty
   public surface before adding behavior.
2. Add failing wire and validation tests in
   `src/request/test.rs` and `src/response/test.rs`; implement the payloads in
   `src/{request,response}/types.rs` and their validators in each `mod.rs`.
3. Add failing configuration, retry, HTTP-status, transport, redirect, secret,
   and failure-metadata tests in `src/client/test.rs`; implement `Client`,
   `ClientConfig`, `RetryPolicy`, `EvaluationResult`, and `EvaluationFailure` in
   `src/client/{mod,types}.rs`. Reject retry counts above 100 and test that
   boundary so the saturating counter cannot become unbounded.
4. Add rendering/source tests in `src/error/test.rs`; implement every classified
   failure in `src/error/mod.rs`. Timeouts/connect failures are retryable;
   request/body/redirect failures are terminal; HTTP status policy is explicit.
5. Update `examples/basic.rs`, `tests/public_api.rs`, module READMEs, root docs,
   `.env.example`, CI, `Cargo.lock`, and `deny.toml` in the same change.
6. Run the verification contract:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo build --all-targets --all-features
   cargo test --all-features
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
   .github/scripts/check-file-coverage.sh 90 coverage.json
   cargo deny check all
   ```

## Completion checklist

- [x] Template and TinyBus artifacts removed.
- [x] Choice, Score, and Noul wires pinned.
- [x] Request-relative response validation implemented.
- [x] HTTPS, redirect, credential, retry, and failure-metadata behavior tested.
- [x] Every production source file exceeds 90% line coverage.
- [x] Format, clippy, build, test, rustdoc, MSRV, supply-chain, and CI checks pass.
