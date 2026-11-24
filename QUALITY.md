# Quality report

This page shows how otel-span-bill was checked. It describes checks we did. It is not a guarantee that the software is free of bugs or that the estimates match an invoice.

## 0.1.0

| | |
|---|---|
| Tested commit | `04ca47690ee893bb9b67b3f2c261bf981615c62c` (first commit of the repair of review B-1 and B-2 on 2026-11-24; the documents on top of it are the only later change) |
| Tests | 155 automated tests (0 failed) |

### What is tested

- 90 unit tests in `src/` (size estimate, counting, input, costs, rendering, prices).
- 50 tests in `tests/cli.rs` that run the binary on the fixtures in `tests/fixtures` and on generated files: exit codes, options, error messages, directories, JSON lines, a byte order mark, binary input, a 20,000-span file, a failed write to stdout, a full or closed stderr, a closed reader, paths with control characters in error messages.
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
- Only Linux was used. The claim "exit 2 without a panic" for a failed stdout and a full or closed stderr is tested on Unix only: those four tests use `/dev/full` or `sh` and have `cfg(unix)`. On Windows they are not compiled, so that claim is not tested there. No GitHub Actions run has happened.

## Run on 2026-11-24 (review B-1 and B-2)

Repair of findings B-1 and B-2 of the final review `otel-eind-6dc6ccb`. N1 to N7 stay open. Built-in prices unchanged. Nothing was pushed, no tag, no GitHub Actions run.

Change (tests only, `04ca47690ee893bb9b67b3f2c261bf981615c62c`): `the_tool_makes_no_network_connection` reads the dependencies in both forms, the `[dependencies]` table and the `[dependencies.NAME]` tables that `cargo package` writes (B-1); `readme_price_table_matches_the_constants` parses the six rows of the README price table and compares price, included amount and source of each row with the constants (B-2).

Red on `6dc6ccb` (a `git archive` copy of that commit, packaged with `cargo package --allow-dirty --offline --no-verify`, unpacked):

```
cargo test --locked --test cli the_tool_makes
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 49 filtered out
```

B-2 on `6dc6ccb`: with `$1.70 per million` changed to `$1.80` and the Grafana write rate to `$0.45` in the table (and both old amounts left elsewhere in the README), `cargo test --locked --test readme` stays `10 passed; 0 failed`. With the new test and the same changed README: `9 passed; 1 failed` (`readme_price_table_matches_the_constants`).

Green on the tip `04ca47690ee893bb9b67b3f2c261bf981615c62c`: `cargo test --locked --no-fail-fast -j 2` in the source tree and in the unpacked `.crate` of that commit, `cargo fmt --check` (exit 0) and `cargo clippy --all-targets --locked -- -D warnings` (exit 0), Linux. The documents commit on top changes only documents.

Result: 155 passed, 0 failed (90 unit tests, 50 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doc-tests), in both trees.

Not run: `cargo test --release`, `cargo +1.85 test`, `cargo deny`, `cargo audit`, a GitHub Actions run.

STAND: groen

## Run on 2026-11-24 (review F1 and F2)

Repair of findings F1 and F2 of the final review `otel-eind-4b7719b`. F3, F4, F5 and the rest stay open. Built-in prices unchanged. Nothing was pushed, no tag.

Change: `cfg(unix)` on `full()` and on the four tests that use `/dev/full` or `sh` (F1); every path in an error message goes through `escape_text` (F2), also for a directory and for a path that does not exist. 3 tests added in `tests/cli.rs` (a path with byte 1b and a line break: no raw escape, one line on stderr).

Red on `4b7719b` (the 3 new tests with the old `src/`):

```
cargo test --locked -j 2 --test cli escape
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 46 filtered out
```

F1 cannot be shown red on Linux: `cfg(unix)` only changes what Windows and macOS compile, and neither was run.

Green on the tip: `cargo test --locked --no-fail-fast -j 2`, `cargo fmt --check` (exit 0) and `cargo clippy --all-targets --locked -- -D warnings` (exit 0), Linux, on the tree of the documents commit directly on top of `04ca47690ee893bb9b67b3f2c261bf981615c62c`. That commit changes only documents.

Result: 155 passed, 0 failed (90 unit tests, 50 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doc-tests).

Not run: `cargo test --release`, `cargo +1.85 test`, `cargo deny`, `cargo audit`, a GitHub Actions run.

STAND: groen

## Run on 2026-11-24 (repair round)

Command: `cargo test --locked --no-fail-fast -j 2`, plus `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`, from the repository root, Linux, on the tree of the documents commit directly on top of `cf8c5d7d3713b2cabfad1270c984c909c1128946`. That commit changes only documents.

Change: a failed write to stdout (anything except a closed reader) now exits 2, and a full or closed stderr exits 2 without a panic (before: exit 0, and exit 101). 5 tests added in `tests/cli.rs`; on the old `src/main.rs` 3 of them fail, the other 2 guard behaviour that was already right.

Result: 152 passed, 0 failed (90 unit tests, 47 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doc-tests).

STAND: groen

## What was not in the short note

This page records checks. It is not a guarantee and it is not a fuzz report.

On 2026-11-18, on this machine, Linux, with cargo -j 2, these commands finished with exit 0: cargo fmt --all -- --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked, and cargo test --release --locked. Debug and release passed the same tests. 0 failed.

Not run that day: cargo deny, cargo audit, a GitHub Actions run, fuzzing, and mutation testing. The CI file lists deny and audit. Those jobs have not run on GitHub for this tree: nothing was pushed.

The Dutch pages are README.nl.md and QUALITY.nl.md. task.md and blueprint.md are local working files: they stay out of git and are listed in .gitignore.

## Run on 2026-11-23

Command: `cargo test --locked --no-fail-fast`, from the repository root, Linux.

Result: 147 passed, 0 failed (90 unit tests, 42 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doc-tests).

STAND: groen

### Not run

- `cargo fmt --check`, `cargo clippy`, `cargo test --release`, `cargo +1.85 test`
- `cargo deny`, `cargo audit`, a GitHub Actions run
- `cargo publish --dry-run`
- fuzzing, property tests and mutation testing

## Run on 2026-11-24

Command: `cargo test --locked --no-fail-fast -j 2`, from the repository root, Linux, on the tree of the documents commit that sits directly on top of `9e53b17`. That commit changes only documents; all code and tests are the ones in `9e53b17`. The README tests read the README at run time, so they need those documents.

Result: 147 passed, 0 failed (90 unit tests, 42 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 doc-tests). The source code did not change in this round. The tests in `tests/readme.rs` now require the full list of README headings.

STAND: groen

### Not run in this round

- `cargo fmt --check`, `cargo clippy`, `cargo test --release`, `cargo +1.85 test`
- `cargo deny`, `cargo audit`, a GitHub Actions run, a release run
- `cargo publish --dry-run`
- fuzzing, property tests and mutation testing
- nothing was pushed to GitHub
