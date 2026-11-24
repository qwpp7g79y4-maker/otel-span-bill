use clap::Parser;
use std::io::{ErrorKind, Write};

/// Write the report to stdout. A closed reader (for example `| head`) is not
/// an error; every other write failure is.
fn write_report(out: &str) -> std::io::Result<()> {
    let mut so = std::io::stdout().lock();
    let r = so.write_all(out.as_bytes()).and_then(|()| so.flush());
    match r {
        Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

/// Report a message on stderr without panicking when stderr itself fails.
fn complain(msg: &str) {
    let _ = writeln!(std::io::stderr().lock(), "error: {msg}");
}

fn main() {
    let cli = otel_span_bill::Cli::parse();
    match otel_span_bill::run(&cli) {
        Ok(out) => {
            if let Err(e) = write_report(&out) {
                complain(&format!("cannot write the report to stdout: {e}"));
                std::process::exit(2);
            }
        }
        Err(e) => {
            complain(&e);
            std::process::exit(2);
        }
    }
}
