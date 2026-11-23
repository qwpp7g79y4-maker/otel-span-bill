//! Find input files and split their text into JSON documents.

use serde_json::{de::StrRead, Deserializer, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Extensions read when a directory is given (case-insensitive).
pub const DIR_EXTENSIONS: [&str; 3] = ["json", "jsonl", "ndjson"];

pub struct Found {
    pub files: Vec<PathBuf>,
    /// Files in a directory that were skipped because of their extension.
    pub skipped: usize,
}

fn has_trace_extension(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| DIR_EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// Expand files and directories. A file is read whatever its name is. A
/// directory is read one level deep, sorted by name, and only for
/// `.json`, `.jsonl` and `.ndjson` files. The same file is read once.
pub fn collect_files(paths: &[PathBuf]) -> Result<Found, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut skipped = 0;
    for p in paths {
        let meta = std::fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
        let mut candidates = Vec::new();
        if meta.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(p)
                .map_err(|e| format!("{}: {e}", p.display()))?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|e| e.is_file())
                .collect();
            entries.sort();
            for e in entries {
                if has_trace_extension(&e) {
                    candidates.push(e);
                } else {
                    skipped += 1;
                }
            }
        } else {
            candidates.push(p.clone());
        }
        for c in candidates {
            let key = std::fs::canonicalize(&c).unwrap_or_else(|_| c.clone());
            if seen.insert(key) {
                out.push(c);
            }
        }
    }
    Ok(Found {
        files: out,
        skipped,
    })
}

/// Split text into JSON documents: one document, JSON lines, or several
/// documents one after the other. A leading byte order mark is ignored.
pub fn documents(text: &str) -> impl Iterator<Item = Result<Value, serde_json::Error>> + '_ {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    Deserializer::new(StrRead::new(text)).into_iter::<Value>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("osb-input-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn one_document() {
        let v: Vec<_> = documents("{\"a\":1}").collect();
        assert_eq!(v.len(), 1);
        assert!(v[0].is_ok());
    }

    #[test]
    fn json_lines() {
        let v: Vec<_> = documents("{\"a\":1}\n{\"a\":2}\n").collect();
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn pretty_printed_documents_back_to_back() {
        let v: Vec<_> = documents("{\n \"a\": 1\n}\n{\n \"a\": 2\n}").collect();
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn blank_lines_are_ignored() {
        let v: Vec<_> = documents("\n\n{\"a\":1}\n\n\n{\"a\":2}\n\n").collect();
        assert_eq!(v.len(), 2);
        assert!(v.iter().all(Result::is_ok));
    }

    #[test]
    fn empty_text_has_no_documents() {
        assert_eq!(documents("").count(), 0);
        assert_eq!(documents("  \n ").count(), 0);
    }

    #[test]
    fn byte_order_mark_is_ignored() {
        let v: Vec<_> = documents("\u{feff}{\"a\":1}").collect();
        assert!(v[0].is_ok());
    }

    #[test]
    fn broken_json_is_an_error() {
        let v: Vec<_> = documents("{\"a\":").collect();
        assert!(v[0].is_err());
    }

    #[test]
    fn directory_reads_json_files_sorted_and_skips_others() {
        let d = tmp("dir");
        fs::write(d.join("b.json"), "{}").unwrap();
        fs::write(d.join("a.jsonl"), "{}").unwrap();
        fs::write(d.join("c.NDJSON"), "{}").unwrap();
        fs::write(d.join("notes.txt"), "x").unwrap();
        let f = collect_files(std::slice::from_ref(&d)).unwrap();
        let names: Vec<_> = f
            .files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.jsonl", "b.json", "c.NDJSON"]);
        assert_eq!(f.skipped, 1);
    }

    #[test]
    fn directory_is_not_read_recursively() {
        let d = tmp("nest");
        fs::create_dir_all(d.join("sub")).unwrap();
        fs::write(d.join("sub").join("x.json"), "{}").unwrap();
        let f = collect_files(&[d]).unwrap();
        assert!(f.files.is_empty());
    }

    #[test]
    fn explicit_file_is_read_whatever_its_name() {
        let d = tmp("file");
        let p = d.join("traces.out");
        fs::write(&p, "{}").unwrap();
        let f = collect_files(std::slice::from_ref(&p)).unwrap();
        assert_eq!(f.files, vec![p]);
    }

    #[test]
    fn same_file_twice_is_read_once() {
        let d = tmp("dup");
        let p = d.join("a.json");
        fs::write(&p, "{}").unwrap();
        let f = collect_files(&[p.clone(), d.clone(), p]).unwrap();
        assert_eq!(f.files.len(), 1);
    }

    #[test]
    fn missing_path_is_an_error_naming_the_path() {
        let e = collect_files(&[PathBuf::from("/nonexistent/otel-span-bill-x")])
            .err()
            .unwrap();
        assert!(e.contains("/nonexistent/otel-span-bill-x"));
    }
}
