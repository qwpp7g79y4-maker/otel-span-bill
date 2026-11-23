//! Build the report and write it as text or JSON.

use crate::cost::{self, Datadog, Grafana, Options};
use crate::model::Aggregate;
use crate::prices::*;
use serde_json::{json, Value};

pub struct ServiceRow {
    pub name: String,
    pub spans: u64,
    pub traces: usize,
    pub bytes: u64,
    /// Share of all estimated bytes, in tenths of a percent.
    pub share_tenths: u64,
    pub datadog_micro: u128,
    pub grafana_micro: u128,
}

pub struct Report {
    pub opts: Options,
    pub files: usize,
    pub skipped_files: usize,
    pub documents: usize,
    pub spans: u64,
    pub traces: usize,
    pub bytes: u64,
    pub service_count: usize,
    pub rows: Vec<ServiceRow>,
    pub omitted: usize,
    pub datadog: Datadog,
    pub grafana: Grafana,
}

/// Totals and per-service rows, biggest service (by spans) first. `top`
/// limits the rows; totals and costs always cover all services.
pub fn build_report(agg: &Aggregate, opts: &Options, top: Option<usize>) -> Report {
    let total_bytes = agg.total_bytes();
    let mut rows: Vec<ServiceRow> = agg
        .services
        .iter()
        .map(|(name, s)| ServiceRow {
            name: name.clone(),
            spans: s.spans,
            traces: s.traces(),
            bytes: s.bytes,
            share_tenths: if total_bytes == 0 {
                0
            } else {
                ((u128::from(s.bytes) * 1000 + u128::from(total_bytes) / 2)
                    / u128::from(total_bytes)) as u64
            },
            datadog_micro: cost::datadog_service_micro(s.bytes, s.spans, opts),
            grafana_micro: cost::grafana_service_micro(s.bytes, opts),
        })
        .collect();
    rows.sort_by(|a, b| b.spans.cmp(&a.spans).then_with(|| a.name.cmp(&b.name)));
    let service_count = rows.len();
    let mut omitted = 0;
    if let Some(n) = top {
        if rows.len() > n {
            omitted = rows.len() - n;
            rows.truncate(n);
        }
    }
    Report {
        opts: *opts,
        files: agg.files,
        skipped_files: agg.skipped_files,
        documents: agg.documents,
        spans: agg.total_spans(),
        traces: agg.total_traces(),
        bytes: total_bytes,
        service_count,
        rows,
        omitted,
        datadog: cost::datadog(total_bytes, agg.total_spans(), agg.hosts.len(), opts),
        grafana: cost::grafana(total_bytes, opts),
    }
}

/// Make a service name safe for a terminal: control characters become `\u{..}`.
pub fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_control() {
            out.push_str(&format!("\\u{{{:x}}}", c as u32));
        } else {
            out.push(c);
        }
    }
    out
}

/// `1234` -> `1.2 KB` (decimal units).
pub fn format_bytes(b: u128) -> String {
    const UNITS: [(&str, u128); 4] = [
        ("TB", 1_000_000_000_000),
        ("GB", 1_000_000_000),
        ("MB", 1_000_000),
        ("KB", 1_000),
    ];
    for (name, size) in UNITS {
        if b >= size {
            let tenths = (b * 10 + size / 2) / size;
            return format!("{}.{} {name}", tenths / 10, tenths % 10);
        }
    }
    format!("{b} B")
}

fn pct(tenths: u64) -> String {
    format!("{}.{}%", tenths / 10, tenths % 10)
}

fn pad_right(s: &str, w: usize) -> String {
    let n = s.chars().count();
    format!("{s}{}", " ".repeat(w.saturating_sub(n)))
}

fn pad_left(s: &str, w: usize) -> String {
    let n = s.chars().count();
    format!("{}{s}", " ".repeat(w.saturating_sub(n)))
}

pub fn render_text(r: &Report) -> String {
    let mut o = String::new();
    o.push_str(&format!("files: {}", r.files));
    if r.skipped_files > 0 {
        o.push_str(&format!(" ({} skipped)", r.skipped_files));
    }
    o.push_str(&format!("  documents: {}\n", r.documents));
    o.push_str(&format!(
        "spans: {}  traces: {}  services: {}  est. size: {}\n",
        r.spans,
        r.traces,
        r.service_count,
        format_bytes(u128::from(r.bytes))
    ));
    o.push_str(&format!("scale: x{}\n\n", r.opts.scale));

    let head = [
        "SERVICE",
        "SPANS",
        "TRACES",
        "EST. SIZE",
        "SHARE",
        "DATADOG",
        "GRAFANA",
    ];
    let body: Vec<[String; 7]> = r
        .rows
        .iter()
        .map(|s| {
            [
                escape_text(&s.name),
                s.spans.to_string(),
                s.traces.to_string(),
                format_bytes(u128::from(s.bytes)),
                pct(s.share_tenths),
                format_usd(s.datadog_micro),
                format_usd(s.grafana_micro),
            ]
        })
        .collect();
    let mut w: Vec<usize> = head.iter().map(|h| h.chars().count()).collect();
    for row in &body {
        for (i, c) in row.iter().enumerate() {
            w[i] = w[i].max(c.chars().count());
        }
    }
    let line = |cells: Vec<&str>| -> String {
        let mut l = String::new();
        for (i, c) in cells.iter().enumerate() {
            if i > 0 {
                l.push_str("  ");
            }
            if i == 0 {
                l.push_str(&pad_right(c, w[i]));
            } else {
                l.push_str(&pad_left(c, w[i]));
            }
        }
        l.trim_end().to_string() + "\n"
    };
    o.push_str(&line(head.to_vec()));
    for row in &body {
        o.push_str(&line(row.iter().map(String::as_str).collect()));
    }
    if r.omitted > 0 {
        o.push_str(&format!("... and {} more services\n", r.omitted));
    }
    o.push_str("DATADOG and GRAFANA per service: usage at list rates, before allotments.\n\n");

    let d = &r.datadog;
    o.push_str("Datadog APM (list prices, billed annually)\n");
    let src = match d.host_source {
        cost::HostSource::Flag => "from --apm-hosts",
        cost::HostSource::Resource => "distinct host.name / host.id in the files",
        cost::HostSource::Assumed => "assumed: no host.name or host.id in the files",
    };
    o.push_str(&format!("  APM hosts: {} ({src})\n", d.hosts));
    o.push_str(&format!(
        "  ingested: {} (included: {})\n",
        format_bytes(d.ingested_bytes),
        format_bytes(d.included_bytes)
    ));
    o.push_str(&format!(
        "  indexed spans: {} at {}% (included: {})\n",
        d.indexed_spans, r.opts.indexed_percent, d.included_indexed
    ));
    o.push_str(&format!(
        "  hosts {} + ingest overage {} + indexed overage {}\n",
        format_usd(d.host_micro),
        format_usd(d.ingest_micro),
        format_usd(d.indexed_micro)
    ));
    o.push_str(&format!("  estimated: {}\n\n", format_usd(d.total_micro)));

    let g = &r.grafana;
    o.push_str("Grafana Cloud Traces (Pro, list prices)\n");
    o.push_str(&format!(
        "  ingested: {} (included: {})\n",
        format_bytes(g.bytes),
        format_bytes(g.included_bytes)
    ));
    o.push_str(&format!(
        "  platform fee {} + processing {} + write {}\n",
        format_usd(g.fee_micro),
        format_usd(g.process_micro),
        format_usd(g.write_micro)
    ));
    o.push_str(&format!("  estimated: {}\n\n", format_usd(g.total_micro)));

    o.push_str(&format!(
        "Prices read on {PRICES_READ_ON} from {DATADOG_URL} and {GRAFANA_URL}. Sizes are an estimate of the OTLP protobuf size.\n"
    ));
    o
}

/// Clamp to what a JSON number can hold without loss of type.
fn num(v: u128) -> Value {
    json!(u64::try_from(v).unwrap_or(u64::MAX))
}

pub fn render_json(r: &Report) -> String {
    let services: Vec<Value> = r
        .rows
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "spans": s.spans,
                "traces": s.traces,
                "estimated_bytes": s.bytes,
                "share_tenths_percent": s.share_tenths,
                "datadog_usage_micro_usd": num(s.datadog_micro),
                "grafana_usage_micro_usd": num(s.grafana_micro),
            })
        })
        .collect();
    let d = &r.datadog;
    let g = &r.grafana;
    let v = json!({
        "tool": "otel-span-bill",
        "version": env!("CARGO_PKG_VERSION"),
        "prices": {
            "read_on": PRICES_READ_ON,
            "datadog_url": DATADOG_URL,
            "grafana_url": GRAFANA_URL,
        },
        "scale": r.opts.scale,
        "files": r.files,
        "skipped_files": r.skipped_files,
        "documents": r.documents,
        "totals": {
            "spans": r.spans,
            "traces": r.traces,
            "services": r.service_count,
            "estimated_bytes": r.bytes,
        },
        "services": services,
        "services_omitted": r.omitted,
        "datadog": {
            "hosts": d.hosts,
            "hosts_source": d.host_source.as_str(),
            "ingested_bytes": num(d.ingested_bytes),
            "included_bytes": num(d.included_bytes),
            "indexed_percent": r.opts.indexed_percent,
            "indexed_spans": num(d.indexed_spans),
            "included_indexed_spans": num(d.included_indexed),
            "host_micro_usd": num(d.host_micro),
            "ingest_micro_usd": num(d.ingest_micro),
            "indexed_micro_usd": num(d.indexed_micro),
            "total_micro_usd": num(d.total_micro),
        },
        "grafana": {
            "ingested_bytes": num(g.bytes),
            "included_bytes": num(g.included_bytes),
            "platform_fee_micro_usd": num(g.fee_micro),
            "processing_micro_usd": num(g.process_micro),
            "write_micro_usd": num(g.write_micro),
            "total_micro_usd": num(g.total_micro),
        },
    });
    serde_json::to_string_pretty(&v).expect("a Value always serialises") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn agg() -> Aggregate {
        let mut a = Aggregate::default();
        for (svc, n) in [("big", 3), ("small", 1), ("alpha", 1)] {
            let spans: Vec<Value> = (0..n)
                .map(|i| json!({"traceId": format!("0{i}"), "name": "op"}))
                .collect();
            let d = json!({"resourceSpans": [{"resource": {"attributes": [{"key": "service.name", "value": {"stringValue": svc}}]},
                "scopeSpans": [{"spans": spans}]}]});
            a.add_document(&d).unwrap();
        }
        a.files = 1;
        a
    }

    #[test]
    fn rows_sorted_by_spans_then_name() {
        let r = build_report(&agg(), &Options::default(), None);
        let names: Vec<_> = r.rows.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["big", "alpha", "small"]);
    }

    #[test]
    fn top_limits_rows_but_not_totals() {
        let r = build_report(&agg(), &Options::default(), Some(1));
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.omitted, 2);
        assert_eq!(r.spans, 5);
        assert_eq!(r.service_count, 3);
    }

    #[test]
    fn top_zero_shows_no_rows() {
        let r = build_report(&agg(), &Options::default(), Some(0));
        assert!(r.rows.is_empty());
        assert_eq!(r.omitted, 3);
    }

    #[test]
    fn top_larger_than_services_omits_nothing() {
        let r = build_report(&agg(), &Options::default(), Some(10));
        assert_eq!(r.omitted, 0);
    }

    #[test]
    fn shares_add_to_about_100_percent() {
        let r = build_report(&agg(), &Options::default(), None);
        let sum: u64 = r.rows.iter().map(|s| s.share_tenths).sum();
        assert!((998..=1002).contains(&sum), "{sum}");
    }

    #[test]
    fn share_of_empty_input_is_zero() {
        let r = build_report(&Aggregate::default(), &Options::default(), None);
        assert!(r.rows.is_empty());
        assert_eq!(r.bytes, 0);
    }

    #[test]
    fn escape_text_escapes_controls_only() {
        assert_eq!(escape_text("a\u{1b}[31mb"), "a\\u{1b}[31mb");
        assert_eq!(escape_text("tab\there"), "tab\\u{9}here");
        assert_eq!(escape_text("é ü 日本"), "é ü 日本");
    }

    #[test]
    fn bytes_are_formatted_in_decimal_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(1_000), "1.0 KB");
        assert_eq!(format_bytes(1_234), "1.2 KB");
        assert_eq!(format_bytes(1_500_000), "1.5 MB");
        assert_eq!(format_bytes(2_000_000_000), "2.0 GB");
        assert_eq!(format_bytes(3_000_000_000_000), "3.0 TB");
    }

    #[test]
    fn text_has_both_vendors_and_the_price_date() {
        let t = render_text(&build_report(&agg(), &Options::default(), None));
        assert!(t.contains("Datadog APM"));
        assert!(t.contains("Grafana Cloud Traces"));
        assert!(t.contains(PRICES_READ_ON));
    }

    #[test]
    fn text_says_when_the_host_count_is_assumed() {
        let t = render_text(&build_report(&agg(), &Options::default(), None));
        assert!(t.contains("assumed"));
    }

    #[test]
    fn text_mentions_omitted_services() {
        let t = render_text(&build_report(&agg(), &Options::default(), Some(1)));
        assert!(t.contains("... and 2 more services"));
    }

    #[test]
    fn json_is_valid_and_has_the_sections() {
        let j = render_json(&build_report(&agg(), &Options::default(), None));
        let v: Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["totals"]["spans"], 5);
        assert_eq!(v["services"].as_array().unwrap().len(), 3);
        assert_eq!(v["datadog"]["total_micro_usd"], 31_000_000);
        assert_eq!(v["grafana"]["total_micro_usd"], 19_000_000);
    }

    #[test]
    fn json_service_names_are_not_escaped_for_the_terminal() {
        let mut a = Aggregate::default();
        let d = json!({"resourceSpans": [{"resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "a\u{1b}b"}}]},
            "scopeSpans": [{"spans": [{"name": "x"}]}]}]});
        a.add_document(&d).unwrap();
        let j = render_json(&build_report(&a, &Options::default(), None));
        let v: Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["services"][0]["name"], "a\u{1b}b");
        assert!(!j.contains('\u{1b}'));
    }
}
