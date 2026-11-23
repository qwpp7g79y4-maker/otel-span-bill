# otel-span-bill

See how many spans each service sends, and what that costs at Datadog APM and Grafana Cloud Traces, from OTLP-JSON files on your machine. Offline.

The tests and the measured numbers are in `QUALITY.md`.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license) [![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue.svg)](Cargo.toml)

[Why](#why) • [Install](#install) • [Quick start](#quick-start) • [How it counts](#how-it-counts) • [Limitations](#limitations) • [Prices](#prices)

otel-span-bill is a command-line tool written in Rust. It reads OpenTelemetry trace data that was saved as OTLP-JSON (for example by the OpenTelemetry Collector `file` exporter), counts spans, traces, and an estimate of the data size per service, and estimates the monthly bill at Datadog APM and Grafana Cloud Traces. It reads files only and opens no network connection. 147 automated tests check it, see [QUALITY.md](QUALITY.md).

## Why

Both vendors charge for trace volume (Datadog also per APM host). One chatty service can be most of the bill, and the invoice does not tell you which one. Saved trace files tell you before you send the data: which service produces the most spans, how large that data is, and what it would cost at the list prices.

## Install

Version 0.1.0. Rust 1.85 or newer is needed. From a checkout of this repository:

```sh
cargo install --locked --path .
otel-span-bill --help
```

After the first release on crates.io: `cargo install --locked otel-span-bill`.

## Quick start

These are the real outputs of the binary on the synthetic fixtures in `tests/fixtures`. A test runs every example below and compares the output.

Two services in one file. `payments` joins a trace that `checkout` started, so there are 2 traces in total, not 3:

```console
$ otel-span-bill tests/fixtures/basic.json
files: 1  documents: 1
spans: 5  traces: 2  services: 2  est. size: 563 B
scale: x1

SERVICE   SPANS  TRACES  EST. SIZE  SHARE  DATADOG  GRAFANA
checkout      3       2      352 B  62.5%    $0.00    $0.00
payments      2       1      211 B  37.5%    $0.00    $0.00
DATADOG and GRAFANA per service: usage at list rates, before allotments.

Datadog APM (list prices, billed annually)
  APM hosts: 2 (distinct host.name / host.id in the files)
  ingested: 563 B (included: 300.0 GB)
  indexed spans: 0 at 0% (included: 2000000)
  hosts $62.00 + ingest overage $0.00 + indexed overage $0.00
  estimated: $62.00

Grafana Cloud Traces (Pro, list prices)
  ingested: 563 B (included: 50.0 GB)
  platform fee $19.00 + processing $0.00 + write $0.00
  estimated: $19.00

Prices read on 2026-11-09 from https://www.datadoghq.com/pricing/list/ and https://grafana.com/pricing/. Sizes are an estimate of the OTLP protobuf size.
```

A directory is read one level deep (`.json`, `.jsonl`, `.ndjson`); here `notes.txt` is skipped. `--top 2` shows the two biggest services; the totals still cover all four:

```console
$ otel-span-bill tests/fixtures/dir --top 2
files: 2 (1 skipped)  documents: 3
spans: 8  traces: 5  services: 4  est. size: 742 B
scale: x1

SERVICE   SPANS  TRACES  EST. SIZE  SHARE  DATADOG  GRAFANA
checkout      3       2      352 B  47.4%    $0.00    $0.00
payments      2       1      211 B  28.4%    $0.00    $0.00
... and 2 more services
DATADOG and GRAFANA per service: usage at list rates, before allotments.

Datadog APM (list prices, billed annually)
  APM hosts: 2 (distinct host.name / host.id in the files)
  ingested: 742 B (included: 300.0 GB)
  indexed spans: 0 at 0% (included: 2000000)
  hosts $62.00 + ingest overage $0.00 + indexed overage $0.00
  estimated: $62.00

Grafana Cloud Traces (Pro, list prices)
  ingested: 742 B (included: 50.0 GB)
  platform fee $19.00 + processing $0.00 + write $0.00
  estimated: $19.00

Prices read on 2026-11-09 from https://www.datadoghq.com/pricing/list/ and https://grafana.com/pricing/. Sizes are an estimate of the OTLP protobuf size.
```

The files are a sample. If the sample is 1/400,000,000 of a month, `--scale 400000000` shows the monthly bill. `--apm-hosts 1` sets the Datadog host count, and `--indexed-percent 10` says that 10% of the spans are kept as indexed spans:

```console
$ otel-span-bill tests/fixtures/basic.json --scale 400000000 --apm-hosts 1 --indexed-percent 10
files: 1  documents: 1
spans: 5  traces: 2  services: 2  est. size: 563 B
scale: x400000000

SERVICE   SPANS  TRACES  EST. SIZE  SHARE  DATADOG  GRAFANA
checkout      3       2      352 B  62.5%  $218.08   $63.36
payments      2       1      211 B  37.5%  $144.44   $37.98
DATADOG and GRAFANA per service: usage at list rates, before allotments.

Datadog APM (list prices, billed annually)
  APM hosts: 1 (from --apm-hosts)
  ingested: 225.2 GB (included: 150.0 GB)
  indexed spans: 200000000 at 10% (included: 1000000)
  hosts $31.00 + ingest overage $7.52 + indexed overage $338.30
  estimated: $376.82

Grafana Cloud Traces (Pro, list prices)
  ingested: 225.2 GB (included: 50.0 GB)
  platform fee $19.00 + processing $8.76 + write $70.08
  estimated: $97.84

Prices read on 2026-11-09 from https://www.datadoghq.com/pricing/list/ and https://grafana.com/pricing/. Sizes are an estimate of the OTLP protobuf size.
```

`--json` writes the same result as JSON, with money in micro-dollars (1 USD = 1,000,000) so nothing is rounded.

## How it counts

- The input is OTLP-JSON, the `ExportTraceServiceRequest` shape with `resourceSpans`. A file can hold one document, JSON lines, or several documents one after the other. `snake_case` names and the older `instrumentationLibrarySpans` are read too. [test: one_document, json_lines, pretty_printed_documents_back_to_back, snake_case_document_is_read, legacy_instrumentation_library_spans_are_read]
- A directory is read one level deep, sorted by name, for `.json`, `.jsonl` and `.ndjson` files; other files are counted as skipped. A file given by name is read whatever its extension. The same file given twice is read once. [test: directory_reads_json_files_sorted_and_skips_others, directory_is_not_read_recursively, explicit_file_is_read_whatever_its_name, the_same_file_twice_is_counted_once]
- Every span counts once, for the service in the `service.name` string attribute of its resource. No `service.name`, an empty one, or one that is not a string gives `unknown_service`. [test: counts_spans_per_service, missing_service_name_is_unknown_service, empty_service_name_is_unknown_service, a_service_name_set_as_a_number_is_unknown_service]
- Traces are distinct trace IDs, compared case-insensitively. A service row counts the traces it took part in; the total counts a trace once. [test: traces_are_distinct_ids, trace_ids_compare_case_insensitively, a_trace_across_two_services_counts_once_in_total]
- The size of a span is the size of its OTLP protobuf `Span` message, with its tag and length in `ScopeSpans`, as proto3 writes it (fields at their default are left out). The `Resource` and `InstrumentationScope` of each block are added once per block. This is checked against a real protobuf encoder. [test: rich_span_matches_the_protobuf_encoder, resource_and_scope_header_match_the_protobuf_encoder, empty_span_is_two_bytes, zero_timestamps_are_omitted]
- One GB is 10^9 bytes. [test: one_gb_at_ten_cents]
- Datadog APM is the number of APM hosts times $31, plus $0.10 per GB of ingested spans above 150 GB per host, plus $1.70 per million indexed spans above 1,000,000 indexed spans per host. [test: datadog_small_sample_costs_one_host, datadog_150_gb_per_host_is_free, datadog_overage_is_ten_cents_per_gb, datadog_allotment_grows_with_hosts, datadog_indexed_overage_per_million, large_scale_crosses_the_gigabyte_allotments]
- The host count is the number of distinct `host.name` values in the resources, or `host.id` where there is no `host.name`. `--apm-hosts N` replaces it. With neither, one host is assumed, and the report says so. [test: host_name_is_collected_and_preferred_over_host_id, host_id_is_used_without_host_name, same_host_twice_counts_once, datadog_flag_overrides_found_hosts, text_without_host_attributes_says_one_host_is_assumed]
- Indexed spans are `--indexed-percent` of the spans, rounded half up. The default is 0, and the maximum is 100. [test: datadog_default_indexes_nothing, indexed_spans_round_half_up, indexed_percent_over_100_is_rejected, indexed_percent_adds_indexed_span_cost]
- Grafana Cloud Traces is the $19 platform fee, plus $0.05 processing and $0.40 write per GB above the 50 GB included. [test: grafana_small_sample_costs_the_platform_fee, grafana_first_50_gb_are_included, grafana_overage_is_processing_plus_write]
- `--scale N` says the files hold 1/N of a month. It multiplies sizes and span counts in the cost lines; the totals line keeps what is in the files. [test: scale_multiplies_sizes_and_counts, scale_multiplies_bytes, scale_zero_is_rejected]
- The DATADOG and GRAFANA columns per service are that service's usage at list rates, before allotments and fees, so the columns do not add up to the estimates. [test: service_costs_use_list_rates_without_allotments, service_datadog_cost_includes_indexed_spans]
- Services are sorted by spans, biggest first, then by name. `--top N` shows only the first N rows and leaves the totals alone. [test: rows_sorted_by_spans_then_name, top_limits_the_rows_and_keeps_the_totals, top_zero_shows_no_rows]
- Money is whole micro-dollars, and cents are rounded half up when shown. [test: rounds_half_up, usd_formatting, totals_do_not_overflow]
- The text output escapes control characters in service names; the JSON output keeps the name as it is. [test: control_characters_in_a_service_name_are_escaped_in_text, json_output_keeps_the_service_name_exactly]
- A path that does not exist, invalid JSON, JSON that is not OTLP trace data, and a directory without trace files all exit with 2, name the file, and print nothing on stdout. [test: missing_path_exits_2_and_names_the_path, invalid_json_exits_2_with_file_and_position, json_that_is_not_otlp_traces_exits_2, directory_without_json_files_exits_2, a_good_file_and_a_bad_file_still_fail]
- The same input always gives the same output. [test: output_is_identical_between_runs]
- It makes no network connection: the only dependencies are `clap` and `serde_json`, and the sources use no socket or HTTP client. [test: the_tool_makes_no_network_connection]

## Limitations

- The sizes are an estimate. Neither Datadog nor Grafana says which byte count it bills, so the OTLP protobuf size is an assumption, and an invoice can differ. [test: the_text_report_says_sizes_are_an_estimate]
- Only list prices are used: Datadog APM billed annually, no APM Pro or Enterprise, no month-to-month or on-demand rates, no Fargate, no contract discounts; Grafana Cloud Traces Pro with no volume discounts and no retention beyond 30 days. [test: the_text_report_names_the_price_sources_and_date, datadog_small_sample_costs_one_host, grafana_small_sample_costs_the_platform_fee]
- Grafana is estimated as if every processed byte is written. Sampling or Adaptive Telemetry that drops data before it is written is not modelled. [test: grafana_overage_is_processing_plus_write]
- Datadog indexed spans depend on your retention filters, which are not in the files. The default is 0 indexed spans; use `--indexed-percent`. [test: datadog_default_indexes_nothing]
- The host count comes from resource attributes. Datadog meters APM hosts hourly and bills a high percentile, so its count can differ. [test: datadog_counts_hosts_from_resources]
- Only OTLP-JSON is read. A binary protobuf file is rejected. [test: a_binary_protobuf_file_is_rejected]
- Only trace data is read. OTLP-JSON with metrics or logs is rejected. [test: json_that_is_not_otlp_traces_exits_2]
- `service.name` is read only as a string attribute. [test: attr_string_ignores_non_string_values]
- Files are read whole into memory; there is no streaming mode. A file of 20,000 spans is tested. [test: a_20000_span_file_is_processed]
- The files do not say what period they cover, so you give `--scale` yourself. [test: scale_multiplies_sizes_and_counts]
- There is no OTLP receiver or other network client in 0.1; it reads files. [test: the_tool_makes_no_network_connection]
- Prices change. They are fixed in the program and dated in the report. Check the pages before you decide anything. [test: the_json_report_names_the_price_sources_and_date]

## Prices

Read on 2026-11-09 from the vendor pages. Datadog does not show an update date on these pages, and neither does Grafana.

| | Price | Included | Source |
|---|---|---|---|
| Datadog APM host, billed annually | $31 per host per month | 150 GB of ingested spans and 1,000,000 indexed spans per host per month | https://www.datadoghq.com/pricing/list/ and https://docs.datadoghq.com/account_management/billing/apm_tracing_profiler/ |
| Datadog ingested spans above that | $0.10 per GB per month | | https://www.datadoghq.com/pricing/list/ |
| Datadog indexed spans above that (15-day retention, billed annually) | $1.70 per million per month | | https://www.datadoghq.com/pricing/list/ |
| Grafana Cloud Pro platform fee | $19 per month | 50 GB per month | https://grafana.com/pricing/ |
| Grafana Cloud Traces processing | $0.05 per GB | | https://grafana.com/pricing/ and https://grafana.com/docs/grafana-cloud/platform/pricing-and-usage/traces/ |
| Grafana Cloud Traces write | $0.40 per GB | | https://grafana.com/pricing/ and https://grafana.com/docs/grafana-cloud/platform/pricing-and-usage/traces/ |

Grafana shows these as "Starts at" prices with automatic volume discounts. A test compares this table with the prices in the program.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.
