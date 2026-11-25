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

/// The Rust runtime reopens a closed fd 1 as /dev/null before `main` runs, so
/// the report would vanish with exit 0. On Linux an `.init_array` hook runs
/// before that and records whether fd 1 was open at process start.
#[cfg(target_os = "linux")]
mod start {
    use std::sync::atomic::{AtomicBool, Ordering};

    static STDOUT_WAS_CLOSED: AtomicBool = AtomicBool::new(false);

    extern "C" {
        fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    }

    const F_GETFD: i32 = 1;

    extern "C" fn check_stdout() {
        // SAFETY: F_GETFD only reads the descriptor flags of fd 1.
        if unsafe { fcntl(1, F_GETFD) } == -1 {
            STDOUT_WAS_CLOSED.store(true, Ordering::Relaxed);
        }
    }

    #[used]
    #[link_section = ".init_array"]
    static INIT: extern "C" fn() = check_stdout;

    pub fn stdout_is_open() -> bool {
        !STDOUT_WAS_CLOSED.load(Ordering::Relaxed)
    }
}

#[cfg(not(target_os = "linux"))]
mod start {
    pub fn stdout_is_open() -> bool {
        true
    }
}

fn main() {
    if !start::stdout_is_open() {
        complain("stdout is closed");
        std::process::exit(2);
    }
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
