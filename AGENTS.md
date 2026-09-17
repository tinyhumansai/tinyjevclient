# Repository Guidelines

`tinyjevclient` is a Rust 2024 client for TypeSafe AI's System One API and Jev
model. `CLAUDE.md` is a symlink to this file.

## Workflow

- Work in a feature worktree created with `worktree <slug>`; never edit `main`.
- Preserve unrelated changes and never rewrite or squash published history.
- The auto-commit hook may checkpoint files. Do not reset those commits.
- Push feature branches and open ready-for-review PRs against the canonical
  `tinyhumansai/tinyjevclient` repository. Do not merge without authorization.
- Never commit API keys. Live calls read `TYPESAFE_API_KEY` only.

## Architecture

The virtual workspace contains one library crate at `crates/tinyjevclient`.
Feature directories under `src/` contain a `mod.rs`, substantial definitions in
`types.rs`, unit tests in `test.rs`, and a short `README.md`:

- `client/`: HTTP execution, retries, secret handling, and measurements.
- `request/`: Choice, Score, Noul, and request validation.
- `response/`: answers and request-relative response validation.
- `error/`: the crate-wide `Error` and `Result` types.

The model supplies typed judgments; application code owns policy, thresholds,
workflow control, and side effects. Do not make confidence an authorization
decision, ask Jev to generate free-form text, or hide conditional workflows in
question text.

## Build and Test

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

CI also runs rustdoc with warnings denied, the declared MSRV build,
`cargo-deny`, and the per-file 90% line-coverage gate. The bundled example is a
paid live operation and must only be run explicitly with `TYPESAFE_API_KEY`.

## Rust Style and Errors

- Use rustfmt and Rust 2024 idioms; unsafe code is forbidden.
- Public items require rustdoc. Fallible public functions require `# Errors`.
- Return the crate-wide `Result<T>` and add specific `Error` variants.
- Do not use `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` in library
  paths.
- Keep files below 700 lines and Markdown below 500 lines.
- Pin serde wire representations in tests and cover every failure variant.
- Tests are deterministic and offline unless explicitly named and gated live.

## Dependencies

Dependencies use caret ranges, minimal features, and a comment in the root
workspace manifest explaining why they exist. Prefer the standard library or an
existing dependency. Keep `Cargo.lock` committed and run `cargo deny check all`
when available.

## Documentation

Keep `README.md`, crate docs, module READMEs, `docs/specs/`, and `docs/plans/`
aligned with behavior. Specs define accepted behavior; plans define delivery
order. The live TypeSafe documentation is the source of truth for API fields and
limits.
