use clap::Parser;
use std::io::Write;

fn main() {
    let cli = otel_span_bill::Cli::parse();
    match otel_span_bill::run(&cli) {
        Ok(out) => {
            // A closed pipe (for example `| head`) is not an error.
            let _ = std::io::stdout().write_all(out.as_bytes());
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}
