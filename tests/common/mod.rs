#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run the binary from the crate root, so fixture paths are relative.
pub fn run(args: &[&str]) -> Out {
    let o = Command::new(env!("CARGO_BIN_EXE_otel-span-bill"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("binary runs");
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

pub fn json(args: &[&str]) -> serde_json::Value {
    let mut a = args.to_vec();
    a.push("--json");
    let o = run(&a);
    assert_eq!(o.code, 0, "{}", o.stderr);
    serde_json::from_str(&o.stdout).expect("valid JSON")
}

static N: AtomicUsize = AtomicUsize::new(0);

/// A fresh empty directory under the cargo target tmp dir.
pub fn tmp_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "t{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub fn write(name: &str, bytes: &[u8]) -> String {
    let p = tmp_dir().join(name);
    std::fs::write(&p, bytes).unwrap();
    p.to_string_lossy().into_owned()
}

/// A one-service document with `n` spans, each in its own trace.
pub fn doc(service: &str, n: usize) -> String {
    let spans: Vec<String> = (0..n)
        .map(|i| format!(r#"{{"traceId":"{i:032x}","spanId":"{i:016x}","name":"op"}}"#))
        .collect();
    format!(
        r#"{{"resourceSpans":[{{"resource":{{"attributes":[{{"key":"service.name","value":{{"stringValue":{}}}}}]}},"scopeSpans":[{{"spans":[{}]}}]}}]}}"#,
        serde_json::to_string(service).unwrap(),
        spans.join(",")
    )
}
