# Implement the System One client

Linked specification: [`../specs/system-one-client.md`](../specs/system-one-client.md).

1. Replace the TinyBus template with the ordinary library at
   `crates/tinyjevclient/`; remove the obsolete module crate, contract crate,
   submodule, packaging workflow, and release documentation.
2. Define and pin Choice, Score, Noul, request, answer, usage, and response
   wires under `crates/tinyjevclient/src/{request,response}/`.
3. Validate requests and request-relative response invariants in those module
   roots, with every case in their adjacent `test.rs` files.
4. Implement the rustls client, secret redaction, classified failures, failure
   measurements, HTTPS policy, and bounded retries under
   `crates/tinyjevclient/src/{client,error}/`.
5. Update the crate example, public API test, root documentation, CI, lockfile,
   dependency policy, and environment example in the same change.
6. Verify with:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo build --all-targets --all-features
   cargo test --all-features
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
   .github/scripts/check-file-coverage.sh 90 coverage.json
   cargo deny check all
   ```
