//! The README, QUALITY.md, CHANGELOG and licences are checked against the code.

mod common;
use otel_span_bill::prices::*;
use std::fs;

fn root(p: &str) -> String {
    format!("{}/{p}", env!("CARGO_MANIFEST_DIR"))
}

fn read(p: &str) -> String {
    fs::read_to_string(root(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
}

/// All Rust sources of the crate: src/ and tests/.
fn sources() -> Vec<(String, String)> {
    let mut out = vec![];
    for dir in ["src", "tests"] {
        let mut stack = vec![std::path::PathBuf::from(root(dir))];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    if p.file_name().unwrap() != "fixtures" {
                        stack.push(p);
                    }
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push((p.display().to_string(), fs::read_to_string(&p).unwrap()));
                }
            }
        }
    }
    out
}

/// Number of test functions in the crate: lines that are exactly the test attribute.
fn test_count() -> usize {
    sources()
        .iter()
        .flat_map(|(_, t)| t.lines())
        .filter(|l| l.trim() == "#[test]")
        .count()
}

fn section<'a>(text: &'a str, heading: &str) -> &'a str {
    let start = text
        .find(&format!("\n## {heading}\n"))
        .unwrap_or_else(|| panic!("no section {heading}"))
        + 1;
    let rest = &text[start..];
    let end = rest[3..].find("\n## ").map(|i| i + 3).unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn readme_has_the_required_sections() {
    let r = read("README.md");
    for h in [
        "Why",
        "Installation",
        "Quick start",
        "Aan de slag in 5 minuten",
        "Options",
        "Example",
        "How it counts",
        "Exit codes",
        "Limitations",
        "Maintenance",
        "Prices and responsibility",
        "License",
    ] {
        assert!(r.contains(&format!("\n## {h}\n")), "missing section {h}");
    }
}

#[test]
fn readme_and_quality_state_the_number_of_tests_in_the_suite() {
    let n = test_count();
    assert!(n >= 60, "{n}");
    for f in ["README.md", "QUALITY.md"] {
        let t = read(f);
        assert!(
            t.contains(&format!("{n} automated tests")),
            "{f} does not say \"{n} automated tests\""
        );
    }
}

#[test]
fn every_claim_in_how_it_counts_and_limitations_names_its_tests() {
    let r = read("README.md");
    for h in ["How it counts", "Limitations"] {
        let items: Vec<&str> = section(&r, h)
            .lines()
            .filter(|l| l.starts_with("- "))
            .collect();
        assert!(items.len() >= 5, "{h} has only {} claims", items.len());
        for i in items {
            assert!(i.contains("[test: "), "no test marker in: {i}");
        }
    }
}

fn marked_tests(readme: &str) -> Vec<String> {
    let mut names = vec![];
    for line in readme.lines() {
        let mut rest = line;
        while let Some(i) = rest.find("[test: ") {
            let after = &rest[i + 7..];
            let end = after.find(']').expect("unclosed [test: marker");
            names.extend(after[..end].split(',').map(|s| s.trim().to_string()));
            rest = &after[end..];
        }
    }
    names
}

#[test]
fn every_named_test_exists_exactly_once() {
    let src = sources();
    let names = marked_tests(&read("README.md"));
    assert!(names.len() >= 20, "{}", names.len());
    for n in names {
        let needle = format!("fn {n}(");
        let hits: usize = src.iter().map(|(_, t)| t.matches(&needle).count()).sum();
        assert_eq!(hits, 1, "test {n} is defined {hits} times");
    }
}

#[test]
fn quick_start_examples_match_the_real_output() {
    let r = read("README.md");
    let qs = section(&r, "Quick start");
    let mut checked = 0;
    let mut lines = qs.lines();
    while let Some(l) = lines.next() {
        if let Some(cmd) = l.strip_prefix("$ otel-span-bill ") {
            let mut want = String::new();
            for o in lines.by_ref() {
                if o == "```" {
                    break;
                }
                want.push_str(o);
                want.push('\n');
            }
            let args: Vec<&str> = cmd.split_whitespace().collect();
            let got = common::run(&args);
            assert_eq!(got.code, 0, "{cmd}: {}", got.stderr);
            assert_eq!(
                got.stdout, want,
                "output of `{cmd}` differs from the README"
            );
            checked += 1;
        }
    }
    assert!(checked >= 3, "only {checked} examples checked");
}

#[test]
fn readme_price_table_matches_the_constants() {
    let r = read("README.md");
    let money = |m: u64| format_usd(u128::from(m));
    for want in [
        money(DD_APM_HOST_MICRO),
        format!("${}", "0.10"),
        format!("${}", "1.70"),
        money(GC_PLATFORM_FEE_MICRO),
        "$0.05".to_string(),
        "$0.40".to_string(),
        format!("{DD_INCLUDED_GB_PER_HOST} GB"),
        format!("{GC_INCLUDED_GB} GB"),
        "1,000,000 indexed spans".to_string(),
        DATADOG_URL.to_string(),
        DATADOG_DOCS_URL.to_string(),
        GRAFANA_URL.to_string(),
        GRAFANA_DOCS_URL.to_string(),
        PRICES_READ_ON.to_string(),
    ] {
        assert!(r.contains(&want), "README does not contain {want}");
    }
    assert_eq!(DD_INGEST_GB_MICRO, 100_000);
    assert_eq!(DD_INDEXED_PER_MILLION_MICRO, 1_700_000);
    assert_eq!(GC_PROCESS_GB_MICRO, 50_000);
    assert_eq!(GC_WRITE_GB_MICRO, 400_000);
    assert_eq!(DD_INCLUDED_INDEXED_PER_HOST, 1_000_000);
}

#[test]
fn readme_install_command_uses_the_crate_name() {
    let r = read("README.md");
    let cargo = read("Cargo.toml");
    assert!(cargo.contains("name = \"otel-span-bill\""));
    assert!(section(&r, "Installation").contains("cargo install --locked otel-span-bill"));
}

#[test]
fn both_licences_are_present_and_cargo_names_them() {
    assert!(read("LICENSE-MIT").starts_with("MIT License"));
    assert!(read("LICENSE-APACHE").contains("Apache License"));
    assert!(read("Cargo.toml").contains("license = \"MIT OR Apache-2.0\""));
    assert!(read("README.md").contains("MIT OR Apache-2.0"));
}

#[test]
fn changelog_has_the_release_of_the_crate_version() {
    let c = read("CHANGELOG.md");
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    assert!(c.contains("## [0.1.0]"));
}

#[test]
fn quality_md_names_the_checks_it_ran() {
    let q = read("QUALITY.md");
    for s in [
        "cargo fmt --check",
        "cargo clippy --all-targets -- -D warnings",
        "cargo test",
        "cargo publish --dry-run",
    ] {
        assert!(q.contains(s), "QUALITY.md does not mention {s}");
    }
}
