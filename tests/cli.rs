mod common;
use common::*;

const BASIC: &str = "tests/fixtures/basic.json";
const LINES: &str = "tests/fixtures/lines.jsonl";
const DIR: &str = "tests/fixtures/dir";

#[test]
fn basic_text_lists_both_services() {
    let o = run(&[BASIC]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o.stdout.contains("checkout"));
    assert!(o.stdout.contains("payments"));
    assert!(o.stdout.contains("spans: 5  traces: 2  services: 2"));
}

#[test]
fn basic_json_totals() {
    let v = json(&[BASIC]);
    assert_eq!(v["totals"]["spans"], 5);
    assert_eq!(v["totals"]["traces"], 2);
    assert_eq!(v["totals"]["services"], 2);
    assert_eq!(v["services"][0]["name"], "checkout");
    assert_eq!(v["services"][0]["spans"], 3);
    assert_eq!(v["services"][0]["traces"], 2);
    assert_eq!(v["services"][1]["name"], "payments");
}

#[test]
fn basic_json_sizes_add_up() {
    let v = json(&[BASIC]);
    let sum: u64 = v["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["estimated_bytes"].as_u64().unwrap())
        .sum();
    assert_eq!(sum, v["totals"]["estimated_bytes"].as_u64().unwrap());
    assert_eq!(sum, 563);
}

#[test]
fn basic_costs_are_hosts_and_fee() {
    let v = json(&[BASIC]);
    assert_eq!(v["datadog"]["hosts"], 2);
    assert_eq!(v["datadog"]["hosts_source"], "resource");
    assert_eq!(v["datadog"]["total_micro_usd"], 62_000_000);
    assert_eq!(v["grafana"]["total_micro_usd"], 19_000_000);
}

#[test]
fn json_lines_file_reads_every_document() {
    let v = json(&[LINES]);
    assert_eq!(v["documents"], 2);
    assert_eq!(v["totals"]["spans"], 3);
}

#[test]
fn lines_file_without_service_name_is_unknown_service() {
    let o = run(&[LINES]);
    assert!(o.stdout.contains("unknown_service"));
    assert!(o.stdout.contains("cart"));
}

#[test]
fn directory_is_read_and_other_files_are_skipped() {
    let v = json(&[DIR]);
    assert_eq!(v["files"], 2);
    assert_eq!(v["skipped_files"], 1);
    assert_eq!(v["totals"]["spans"], 8);
    let o = run(&[DIR]);
    assert!(o.stdout.contains("files: 2 (1 skipped)"));
}

#[test]
fn two_files_give_the_same_totals_as_their_directory() {
    let files = json(&[BASIC, LINES]);
    let dir = json(&[DIR]);
    assert_eq!(files["totals"], dir["totals"]);
    assert_eq!(files["services"], dir["services"]);
}

#[test]
fn the_same_file_twice_is_counted_once() {
    let once = json(&[BASIC]);
    let twice = json(&[BASIC, BASIC]);
    assert_eq!(once["totals"], twice["totals"]);
    assert_eq!(twice["files"], 1);
}

#[test]
fn missing_path_exits_2_and_names_the_path() {
    let o = run(&["tests/fixtures/does-not-exist.json"]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("does-not-exist.json"));
    assert!(o.stdout.is_empty());
}

#[test]
fn invalid_json_exits_2_with_file_and_position() {
    let o = run(&["tests/fixtures/broken.json"]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("tests/fixtures/broken.json"));
    assert!(o.stderr.contains("line 2"));
    assert!(o.stdout.is_empty());
}

#[test]
fn json_that_is_not_otlp_traces_exits_2() {
    let o = run(&["tests/fixtures/not-otlp.json"]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("resourceSpans"));
    assert!(o.stderr.contains("document 1"));
}

#[test]
fn a_good_file_and_a_bad_file_still_fail() {
    let o = run(&[BASIC, "tests/fixtures/not-otlp.json"]);
    assert_eq!(o.code, 2);
    assert!(o.stdout.is_empty());
}

#[test]
fn directory_without_json_files_exits_2() {
    let d = tmp_dir();
    std::fs::write(d.join("a.txt"), "x").unwrap();
    let o = run(&[d.to_str().unwrap()]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("no OTLP-JSON files found"));
}

#[test]
fn no_arguments_exits_2() {
    let o = run(&[]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("Usage"));
}

#[test]
fn unknown_option_exits_2() {
    assert_eq!(run(&[BASIC, "--bogus"]).code, 2);
}

#[test]
fn help_lists_every_option() {
    let o = run(&["--help"]);
    assert_eq!(o.code, 0);
    for flag in [
        "--json",
        "--top",
        "--scale",
        "--apm-hosts",
        "--indexed-percent",
    ] {
        assert!(o.stdout.contains(flag), "{flag}");
    }
}

#[test]
fn version_flag_prints_the_crate_version() {
    let o = run(&["--version"]);
    assert_eq!(o.code, 0);
    assert!(o.stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn top_limits_the_rows_and_keeps_the_totals() {
    let v = json(&[DIR, "--top", "1"]);
    assert_eq!(v["services"].as_array().unwrap().len(), 1);
    assert_eq!(v["services_omitted"], 3);
    assert_eq!(v["totals"]["services"], 4);
    assert_eq!(v["totals"]["spans"], 8);
    let o = run(&[DIR, "--top", "1"]);
    assert!(o.stdout.contains("... and 3 more services"));
}

#[test]
fn scale_multiplies_sizes_and_counts() {
    let one = json(&[BASIC]);
    let ten = json(&[BASIC, "--scale", "10"]);
    assert_eq!(ten["scale"], 10);
    assert_eq!(
        ten["grafana"]["ingested_bytes"].as_u64().unwrap(),
        one["grafana"]["ingested_bytes"].as_u64().unwrap() * 10
    );
    assert_eq!(
        ten["totals"]["spans"], 5,
        "totals stay what is in the files"
    );
}

#[test]
fn scale_zero_is_rejected() {
    assert_eq!(run(&[BASIC, "--scale", "0"]).code, 2);
}

#[test]
fn apm_hosts_flag_overrides_the_files() {
    let v = json(&[BASIC, "--apm-hosts", "5"]);
    assert_eq!(v["datadog"]["hosts"], 5);
    assert_eq!(v["datadog"]["hosts_source"], "flag");
    assert_eq!(v["datadog"]["total_micro_usd"], 155_000_000);
}

#[test]
fn apm_hosts_zero_is_rejected() {
    assert_eq!(run(&[BASIC, "--apm-hosts", "0"]).code, 2);
}

#[test]
fn indexed_percent_over_100_is_rejected() {
    assert_eq!(run(&[BASIC, "--indexed-percent", "101"]).code, 2);
}

#[test]
fn indexed_percent_adds_indexed_span_cost() {
    // 5 spans x 1,000,000 = 5M indexed; 2 hosts include 2M; 3M x $1.70 = $5.10
    let v = json(&[BASIC, "--scale", "1000000", "--indexed-percent", "100"]);
    assert_eq!(v["datadog"]["indexed_spans"], 5_000_000);
    assert_eq!(v["datadog"]["indexed_micro_usd"], 5_100_000);
    assert_eq!(v["datadog"]["total_micro_usd"], 67_100_000);
}

#[test]
fn large_scale_crosses_the_gigabyte_allotments() {
    // 563 bytes x 400,000,000 = 225.2 GB
    let v = json(&[BASIC, "--scale", "400000000", "--apm-hosts", "1"]);
    assert_eq!(v["datadog"]["ingested_bytes"], 225_200_000_000u64);
    // (225.2 - 150) GB x $0.10 = $7.52 on top of one host at $31
    assert_eq!(v["datadog"]["ingest_micro_usd"], 7_520_000);
    assert_eq!(v["datadog"]["total_micro_usd"], 38_520_000);
    // (225.2 - 50) GB x ($0.05 + $0.40) = $78.84, plus the $19 fee
    assert_eq!(v["grafana"]["processing_micro_usd"], 8_760_000);
    assert_eq!(v["grafana"]["write_micro_usd"], 70_080_000);
    assert_eq!(v["grafana"]["total_micro_usd"], 97_840_000);
}

#[test]
fn text_and_json_agree_on_the_dollar_totals() {
    let args = [BASIC, "--scale", "400000000", "--apm-hosts", "1"];
    let t = run(&args).stdout;
    assert!(t.contains("estimated: $38.52"));
    assert!(t.contains("estimated: $97.84"));
}

#[test]
fn text_without_host_attributes_says_one_host_is_assumed() {
    let f = write("nohost.json", doc("svc", 2).as_bytes());
    let o = run(&[&f]);
    assert!(o.stdout.contains("APM hosts: 1 (assumed"));
}

#[test]
fn control_characters_in_a_service_name_are_escaped_in_text() {
    let f = write("esc.json", doc("evil\u{1b}[31mname", 1).as_bytes());
    let o = run(&[&f]);
    assert_eq!(o.code, 0);
    assert!(!o.stdout.contains('\u{1b}'));
    assert!(o.stdout.contains("evil\\u{1b}[31mname"));
}

#[test]
fn json_output_keeps_the_service_name_exactly() {
    let f = write("esc2.json", doc("evil\u{1b}name", 1).as_bytes());
    let v = json(&[&f]);
    assert_eq!(v["services"][0]["name"], "evil\u{1b}name");
}

#[test]
fn a_binary_protobuf_file_is_rejected() {
    let f = write("trace.pb", &[0x0a, 0x10, 0xff, 0xfe, 0x00, 0x01]);
    let o = run(&[&f]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("trace.pb"));
}

#[test]
fn a_file_with_a_byte_order_mark_is_read() {
    let mut b = vec![0xEF, 0xBB, 0xBF];
    b.extend_from_slice(doc("bom", 2).as_bytes());
    let f = write("bom.json", &b);
    assert_eq!(json(&[&f])["totals"]["spans"], 2);
}

#[test]
fn pretty_printed_documents_back_to_back_are_read() {
    let a = serde_json::to_string_pretty(
        &serde_json::from_str::<serde_json::Value>(&doc("a", 1)).unwrap(),
    )
    .unwrap();
    let b = serde_json::to_string_pretty(
        &serde_json::from_str::<serde_json::Value>(&doc("b", 2)).unwrap(),
    )
    .unwrap();
    let f = write("pretty.json", format!("{a}\n{b}\n").as_bytes());
    let v = json(&[&f]);
    assert_eq!(v["documents"], 2);
    assert_eq!(v["totals"]["spans"], 3);
}

#[test]
fn an_empty_file_has_zero_spans() {
    let f = write("empty.json", b"");
    let v = json(&[&f]);
    assert_eq!(v["totals"]["spans"], 0);
    assert_eq!(v["files"], 1);
    assert_eq!(v["datadog"]["total_micro_usd"], 31_000_000);
}

#[test]
fn snake_case_and_legacy_names_work_end_to_end() {
    let snake = r#"{"resource_spans":[{"resource":{"attributes":[{"key":"service.name","value":{"string_value":"snk"}}]},"scope_spans":[{"spans":[{"trace_id":"01","name":"x"}]}]}]}"#;
    let legacy = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"old"}}]},"instrumentationLibrarySpans":[{"instrumentationLibrary":{"name":"l"},"spans":[{"traceId":"02","name":"x"}]}]}]}"#;
    let a = write("snake.json", snake.as_bytes());
    let b = write("legacy.json", legacy.as_bytes());
    let v = json(&[&a, &b]);
    let names: Vec<_> = v["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"snk".to_string()));
    assert!(names.contains(&"old".to_string()));
}

#[test]
fn a_20000_span_file_is_processed() {
    let f = write("big.json", doc("big", 20_000).as_bytes());
    let v = json(&[&f]);
    assert_eq!(v["totals"]["spans"], 20_000);
    assert_eq!(v["totals"]["traces"], 20_000);
}

#[test]
fn output_is_identical_between_runs() {
    assert_eq!(run(&[DIR]).stdout, run(&[DIR]).stdout);
    assert_eq!(run(&[DIR, "--json"]).stdout, run(&[DIR, "--json"]).stdout);
}

#[test]
fn the_text_report_says_sizes_are_an_estimate() {
    assert!(run(&[BASIC]).stdout.contains("estimate"));
}

#[test]
fn the_text_report_names_the_price_sources_and_date() {
    let t = run(&[BASIC]).stdout;
    assert!(t.contains("2026-11-09"));
    assert!(t.contains("https://www.datadoghq.com/pricing/list/"));
    assert!(t.contains("https://grafana.com/pricing/"));
    assert!(t.contains("billed annually"));
}

#[test]
fn the_json_report_names_the_price_sources_and_date() {
    let v = json(&[BASIC]);
    assert_eq!(v["prices"]["read_on"], "2026-11-09");
    assert_eq!(
        v["prices"]["datadog_url"],
        "https://www.datadoghq.com/pricing/list/"
    );
    assert_eq!(v["prices"]["grafana_url"], "https://grafana.com/pricing/");
}

#[test]
fn a_service_name_set_as_a_number_is_unknown_service() {
    let d = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"intValue":"5"}}]},"scopeSpans":[{"spans":[{"name":"x"}]}]}]}"#;
    let f = write("intname.json", d.as_bytes());
    assert_eq!(json(&[&f])["services"][0]["name"], "unknown_service");
}

#[test]
fn the_tool_makes_no_network_connection() {
    // Only these crates are linked, and neither opens sockets.
    let cargo =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let deps: Vec<&str> = cargo
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert_eq!(deps.len(), 2, "{deps:?}");
    assert!(deps[0].starts_with("clap"));
    assert!(deps[1].starts_with("serde_json"));
    // And the sources do not touch std::net or an HTTP client.
    let mut files = vec![];
    for e in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap() {
        files.push(e.unwrap().path());
    }
    for f in files {
        let t = std::fs::read_to_string(&f).unwrap();
        for banned in [
            "std::net",
            "TcpStream",
            "UdpSocket",
            "reqwest",
            "hyper",
            "ureq",
            "tonic",
        ] {
            assert!(!t.contains(banned), "{} contains {banned}", f.display());
        }
    }
}

fn bin() -> std::process::Command {
    let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_otel-span-bill"));
    c.current_dir(env!("CARGO_MANIFEST_DIR"));
    c
}

fn full() -> std::fs::File {
    std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full")
}

#[test]
fn a_failed_write_to_stdout_exits_2() {
    let o = bin()
        .arg(BASIC)
        .stdout(full())
        .output()
        .expect("binary runs");
    assert_eq!(o.status.code(), Some(2));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("stdout"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
}

#[test]
fn a_failed_write_to_stdout_with_a_full_stderr_exits_2_without_panic() {
    let o = bin()
        .arg(BASIC)
        .stdout(full())
        .stderr(full())
        .status()
        .expect("binary runs");
    assert_eq!(o.code(), Some(2));
}

#[test]
fn a_closed_stdout_reader_stays_silent_and_exits_0() {
    use std::process::Stdio;
    let mut child = bin()
        .arg("tests/fixtures/dir")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary runs");
    drop(child.stdout.take());
    let o = child.wait_with_output().expect("wait");
    // The reader may close before or after the write; either way: no error.
    assert_eq!(o.status.code(), Some(0));
    assert!(
        o.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn a_full_stderr_makes_an_error_exit_2_without_panic() {
    let o = bin()
        .arg("tests/fixtures/does-not-exist.json")
        .stderr(full())
        .output()
        .expect("binary runs");
    assert_eq!(o.status.code(), Some(2));
}

#[test]
fn a_closed_stderr_makes_an_error_exit_2() {
    let o = std::process::Command::new("sh")
        .arg("-c")
        .arg("exec \"$0\" tests/fixtures/does-not-exist.json 2>&-")
        .arg(env!("CARGO_BIN_EXE_otel-span-bill"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("sh runs");
    assert_eq!(o.status.code(), Some(2));
}
