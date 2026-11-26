# Quality report

pep: 8.6

This page shows how otel-span-bill was checked. It describes checks we did. It is not a guarantee that the software is free of bugs or that the estimates match an invoice. The Dutch version is [QUALITY.nl.md](QUALITY.nl.md).

## 0.1.0

| | |
|---|---|
| Tested commit | `5e746f2c556c2874d2ca1f1e74246a2390c0c0bc` (the fix for a closed stdout, on top of `origin/main` `603ad2b`; the commit with this page sits directly on top of it and changes only documents) |
| Date | 2026-11-25 |
| Tests | 157 automated tests (0 failed) |

### What is tested

- 90 unit tests in `src/` (size estimate, counting, input, costs, rendering, prices).
- 52 tests in `tests/cli.rs` that run the binary on the fixtures in `tests/fixtures` and on generated files: exit codes, options, error messages, directories, JSON lines, a byte order mark, binary input, a 20,000-span file, a failed write to stdout, a full or closed stderr, a closed reader, a closed stdout at the start (exit 2) and stdout on /dev/null (exit 0), paths with control characters in error messages.
- 5 tests in `tests/size_golden.rs`: the size estimate equals the byte count of a real protobuf encoder for a span with every kind of field, and for the resource and scope headers.
- 10 tests in `tests/readme.rs`: every command in the README Quick start is run and its output compared; every claim in "How it counts" and "Limitations" names tests that exist; the prices, URLs and date in the README match the program; the test count in README and this page match the suite; licences and changelog are present.

### How we checked

Run on 2026-11-25 from the root of a clean worktree of commit `5e746f2`, Linux, Rust 1.99.0 stable (`rustc 1.99.0 (b940084d7 2026-09-28)`). Nothing in the tree was changed to get these results.

| Command | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0; 157 passed, 0 failed (90 unit, 52 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doc-tests) |
| `cargo test --release --locked` | exit 0; 157 passed, 0 failed (same split) |

Run also on 2026-11-25 on the same code (`git diff 5e746f2 9e3660d` shows no change in `src/`, `tests/`, `Cargo.toml` or `Cargo.lock`; checked out at `9e3660d`), Linux, Rust 1.85.1 (`rustc 1.85.1 (4eb161250 2025-03-15)`), the minimum supported version:

| Command | Result |
|---|---|
| `cargo +1.85 fmt --check` | exit 0 |
| `cargo +1.85 clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo +1.85 test --locked` | exit 0; 157 passed, 0 failed (90 unit, 52 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doc-tests) |

STAND: groen

### Not run

This measurement did not do these. Earlier pages of this file reported some of them for older commits; they were not repeated for `5e746f2`.

- run on 2026-11-25, Rust 1.85.1: `cargo +1.85 test --release --locked` exit 0; 157 passed, 0 failed, 0 ignored (90 unit, 52 in `tests/cli.rs`, 10 in `tests/readme.rs`, 5 in `tests/size_golden.rs`, 0 in `src/main.rs`, 0 doc-tests)
- not run: `cargo publish --dry-run`
- not run: the form `cargo clippy --all-targets -- -D warnings` without `--locked` (the 2026-11-09 page used it; this measurement used the `--locked` form above)
- not run: `cargo deny`, `cargo audit`
- not run: Windows and macOS, and any GitHub Actions run (nothing was pushed)
- not run: fuzzing, property tests, mutation testing

### What we did not check

- The estimates were not compared with a real Datadog or Grafana invoice. The vendor pages do not say which byte count they bill.
- Only Linux was used. The detection of a closed stdout at the start exists on Linux only (other platforms keep exit 0 and no message). The tests that use `/dev/full` or `sh` are compiled on Linux only, so the claim "exit 2 without a panic" for a failed stdout and a full or closed stderr is not tested on Windows or macOS.
- The built-in prices were not re-checked against the vendor pages in this measurement.

`task.md` and `blueprint.md` are local working files: they stay out of git and are listed in `.gitignore`.
