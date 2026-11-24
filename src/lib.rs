//! otel-span-bill: span volume per service from OTLP-JSON files, with an
//! estimate of the Datadog APM and Grafana Cloud Traces bill. Offline.

pub mod cost;
pub mod input;
pub mod model;
pub mod prices;
pub mod render;
pub mod size;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "otel-span-bill",
    version,
    about = "Show OpenTelemetry span volume per service from OTLP-JSON files, with estimated Datadog APM and Grafana Cloud Traces cost. Offline: reads files only.",
    after_help = "Exit codes: 0 ok, 2 error (unreadable path, invalid JSON, not OTLP trace JSON, no input files, bad option)."
)]
pub struct Cli {
    /// OTLP-JSON files, or directories (one level deep: .json, .jsonl, .ndjson)
    #[arg(required = true, value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Write the result as JSON
    #[arg(long)]
    pub json: bool,

    /// Show only the N biggest services (totals still cover all)
    #[arg(long, value_name = "N")]
    pub top: Option<usize>,

    /// The files hold 1/N of a month: multiply sizes and counts by N
    #[arg(long, value_name = "N", default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..=1_000_000_000))]
    pub scale: u64,

    /// Number of Datadog APM hosts (default: distinct host.name / host.id in the files, else 1)
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u64).range(1..=1_000_000))]
    pub apm_hosts: Option<u64>,

    /// Percent of spans kept as Datadog indexed spans (0 to 100)
    #[arg(long, value_name = "P", default_value_t = 0, value_parser = clap::value_parser!(u64).range(0..=100))]
    pub indexed_percent: u64,
}

/// Read every file and return the aggregate.
pub fn read_inputs(paths: &[PathBuf]) -> Result<model::Aggregate, String> {
    let found = input::collect_files(paths)?;
    if found.files.is_empty() {
        return Err(
            "no OTLP-JSON files found (a directory is read for .json, .jsonl and .ndjson files)"
                .to_string(),
        );
    }
    let mut agg = model::Aggregate::default();
    agg.skipped_files = found.skipped;
    for f in &found.files {
        let name = input::shown(f);
        let text = std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", name))?;
        for (i, doc) in input::documents(&text).enumerate() {
            let doc = doc.map_err(|e| format!("{}: {e}", name))?;
            agg.add_document(&doc)
                .map_err(|e| format!("{}: document {}: {e}", name, i + 1))?;
        }
        agg.files += 1;
    }
    Ok(agg)
}

/// Run the tool and return the text for stdout.
pub fn run(cli: &Cli) -> Result<String, String> {
    let agg = read_inputs(&cli.paths)?;
    let opts = cost::Options {
        scale: cli.scale,
        apm_hosts: cli.apm_hosts,
        indexed_percent: cli.indexed_percent,
    };
    let report = render::build_report(&agg, &opts, cli.top);
    Ok(if cli.json {
        render::render_json(&report)
    } else {
        render::render_text(&report)
    })
}
