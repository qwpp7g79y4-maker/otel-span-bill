# Quality report

This page shows how otel-span-bill was checked. It describes checks we did. It is not a guarantee that the software is free of bugs or that the estimates match an invoice.

## 0.1.0

| | |
|---|---|
| Checked tree | the 0.1.0 commit on branch `w4-W-96` |
| Tests | 147 automated tests (0 failed) |

### What is tested

- 90 unit tests in `src/` (size estimate, counting, input, costs, rendering, prices).
- 42 tests in `tests/cli.rs` that run the binary on the fixtures in `tests/fixtures` and on generated files: exit codes, options, error messages, directories, JSON lines, a byte order mark, binary input, a 20,000-span file.
- 5 tests in `tests/size_golden.rs`: the size estimate equals the byte count of a real protobuf encoder (Python `google.protobuf` 3.12 with the `opentelemetry.proto.trace.v1` message layout written out by hand) for a span with every kind of field, and for the resource and scope headers. This check found one bug while writing it: `Span.flags` is field 16 and needs a two-byte tag.
- 10 tests in `tests/readme.rs`: every command in the README Quick start is run and its output compared; every claim in "How it counts" and "Limitations" names tests that exist; the prices, URLs and date in the README match the program; the test count in README and this page match the suite; licences and changelog are present.

### How we checked

Run from the repository root on 2026-11-09, Linux, Rust 1.99 stable unless noted.

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo test --release`
- `cargo +1.85 test` (the minimum supported Rust version)
- `cargo publish --dry-run`

| Command | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0, on Rust 1.99 and on Rust 1.85 |
| `cargo test` | 147 passed, 0 failed |
| `cargo test --release` | 147 passed, 0 failed |
| `cargo +1.85 test` | 147 passed, 0 failed |
| `cargo publish --dry-run` | green (exit 0; the upload step was aborted by the dry run, nothing was published) |

### What we did not check

- No fuzzing and no property tests in 0.1.0.
- The estimates were not compared with a real Datadog or Grafana invoice. The vendor pages do not say which byte count they bill.
- Only Linux was used.

## What was not in the short note

This page records checks. It is not a guarantee and it is not a fuzz report.

On 2026-11-18, on this machine, Linux, with cargo -j 2, these commands finished with exit 0: cargo fmt --all -- --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked, and cargo test --release --locked. Debug and release passed the same tests. 0 failed.

Not run that day: cargo deny, cargo audit, a GitHub Actions run, fuzzing, and mutation testing. The CI file lists deny and audit. Those jobs have not run on GitHub for this tree, because this tree is not pushed.

There is no Dutch README. task.md and blueprint.md belong in the repository when they exist. A gitignore line that hides them is a mistake.

## Run on 2026-11-23

Command: `cargo test --locked --no-fail-fast`, from the repository root, Linux.

Result: 147 passed, 0 failed (90 unit tests, 42 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doc-tests).

STAND: groen

### Not run

- `cargo fmt --check`, `cargo clippy`, `cargo test --release`, `cargo +1.85 test`
- `cargo deny`, `cargo audit`, a GitHub Actions run
- `cargo publish --dry-run`
- fuzzing, property tests and mutation testing
