# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Fixed

- Paths in error messages are escaped, so a file name with a control character cannot add a line or a terminal escape to stderr.
- The tests that need `/dev/full` or `sh` only run on Unix.

- The README tests read the files with CRLF turned into LF, and `.gitattributes` keeps text files on LF, so a CRLF checkout (for example on Windows) no longer breaks five tests.

### Documentation

- Added README.nl.md, QUALITY.nl.md, CONTRIBUTING.md and SECURITY.md.
- README now has the headings Installation, Aan de slag in 5 minuten, Options, Example, Exit codes, Maintenance and Prices and responsibility.
- QUALITY.md names the tested commit and the run of 2026-11-24. No change to the program.

## [0.1.0] - 2026-11-09

First release.

### Added

- `otel-span-bill PATH...` reads OTLP-JSON trace files (one document, JSON lines, or several documents one after the other) and directories (one level deep: `.json`, `.jsonl`, `.ndjson`), and shows spans, traces, and an estimated OTLP protobuf size per service.
- Estimated monthly cost at Datadog APM (hosts, ingested GB above the allotment, indexed spans above the allotment) and Grafana Cloud Traces Pro (platform fee, processing, write), from list prices read on 2026-11-09. The sources and the date are in the README and in every report.
- Options `--json`, `--top N`, `--scale N`, `--apm-hosts N`, `--indexed-percent P`.
- The size estimate is checked against a real protobuf encoder (`tests/size_golden.rs`).
- Offline: reads files only and has no network client.
