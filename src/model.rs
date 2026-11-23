//! Read OTLP-JSON export requests and total spans per service.

use crate::size::{block_header_len, field, schema_url_len, span_len};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Service name used when a resource has no `service.name` (OpenTelemetry convention).
pub const UNKNOWN_SERVICE: &str = "unknown_service";

#[derive(Debug, Default)]
pub struct ServiceStats {
    pub spans: u64,
    /// Estimated OTLP protobuf bytes: spans plus the resource and scope headers.
    pub bytes: u64,
    traces: HashSet<String>,
}

impl ServiceStats {
    pub fn traces(&self) -> usize {
        self.traces.len()
    }
}

#[derive(Debug, Default)]
pub struct Aggregate {
    pub services: BTreeMap<String, ServiceStats>,
    pub hosts: BTreeSet<String>,
    pub files: usize,
    pub skipped_files: usize,
    pub documents: usize,
    all_traces: HashSet<String>,
}

impl Aggregate {
    pub fn total_spans(&self) -> u64 {
        self.services.values().map(|s| s.spans).sum()
    }

    pub fn total_bytes(&self) -> u64 {
        self.services.values().map(|s| s.bytes).sum()
    }

    /// Distinct trace ids over all services.
    pub fn total_traces(&self) -> usize {
        self.all_traces.len()
    }

    /// Add one OTLP `ExportTraceServiceRequest` in JSON form.
    pub fn add_document(&mut self, doc: &Value) -> Result<(), String> {
        let blocks = field(doc, "resourceSpans", "resource_spans")
            .and_then(Value::as_array)
            .ok_or_else(|| "no `resourceSpans` array (is this OTLP trace JSON?)".to_string())?;
        self.documents += 1;
        for rs in blocks {
            self.add_resource_spans(rs);
        }
        Ok(())
    }

    fn add_resource_spans(&mut self, rs: &Value) {
        let resource = rs.get("resource");
        let service = resource
            .and_then(|r| attr_string(r, "service.name"))
            .filter(|s| !s.is_empty())
            .unwrap_or(UNKNOWN_SERVICE)
            .to_string();
        let host = resource
            .and_then(|r| attr_string(r, "host.name").or_else(|| attr_string(r, "host.id")))
            .filter(|h| !h.is_empty());
        if let Some(h) = host {
            self.hosts.insert(h.to_string());
        }
        let mut bytes = block_header_len(
            resource,
            None,
            schema_url_len(field(rs, "schemaUrl", "schema_url")),
        );
        let scopes = field(rs, "scopeSpans", "scope_spans")
            .or_else(|| {
                field(
                    rs,
                    "instrumentationLibrarySpans",
                    "instrumentation_library_spans",
                )
            })
            .and_then(Value::as_array);
        let mut spans = 0u64;
        let mut trace_ids: Vec<String> = Vec::new();
        for ss in scopes.map(|v| v.as_slice()).unwrap_or(&[]) {
            let scope = ss
                .get("scope")
                .or_else(|| field(ss, "instrumentationLibrary", "instrumentation_library"));
            bytes += block_header_len(
                None,
                scope,
                schema_url_len(field(ss, "schemaUrl", "schema_url")),
            );
            for span in ss
                .get("spans")
                .and_then(Value::as_array)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
            {
                spans += 1;
                bytes += span_len(span);
                if let Some(t) = field(span, "traceId", "trace_id").and_then(Value::as_str) {
                    if !t.is_empty() {
                        trace_ids.push(t.to_ascii_lowercase());
                    }
                }
            }
        }
        let stats = self.services.entry(service).or_default();
        stats.spans += spans;
        stats.bytes += bytes;
        for t in trace_ids {
            self.all_traces.insert(t.clone());
            stats.traces.insert(t);
        }
    }
}

/// The string value of a resource attribute, if it is set as `stringValue`.
pub fn attr_string<'a>(obj: &'a Value, key: &str) -> Option<&'a str> {
    obj.get("attributes")?
        .as_array()?
        .iter()
        .find(|kv| kv.get("key").and_then(Value::as_str) == Some(key))?
        .get("value")
        .and_then(|v| field(v, "stringValue", "string_value"))
        .and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(service: Option<&str>, spans: Vec<Value>) -> Value {
        let attrs = match service {
            Some(s) => json!([{"key": "service.name", "value": {"stringValue": s}}]),
            None => json!([]),
        };
        json!({"resourceSpans": [{"resource": {"attributes": attrs}, "scopeSpans": [{"spans": spans}]}]})
    }

    fn span(trace: &str) -> Value {
        json!({"traceId": trace, "spanId": "0102030405060708", "name": "op"})
    }

    #[test]
    fn counts_spans_per_service() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![span("01"), span("02")]))
            .unwrap();
        a.add_document(&doc(Some("b"), vec![span("03")])).unwrap();
        assert_eq!(a.services["a"].spans, 2);
        assert_eq!(a.services["b"].spans, 1);
        assert_eq!(a.total_spans(), 3);
    }

    #[test]
    fn same_service_in_two_documents_adds_up() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![span("01")])).unwrap();
        a.add_document(&doc(Some("a"), vec![span("02")])).unwrap();
        assert_eq!(a.services.len(), 1);
        assert_eq!(a.services["a"].spans, 2);
        assert_eq!(a.documents, 2);
    }

    #[test]
    fn missing_service_name_is_unknown_service() {
        let mut a = Aggregate::default();
        a.add_document(&doc(None, vec![span("01")])).unwrap();
        assert!(a.services.contains_key(UNKNOWN_SERVICE));
    }

    #[test]
    fn empty_service_name_is_unknown_service() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some(""), vec![span("01")])).unwrap();
        assert!(a.services.contains_key(UNKNOWN_SERVICE));
    }

    #[test]
    fn traces_are_distinct_ids() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![span("01"), span("01"), span("02")]))
            .unwrap();
        assert_eq!(a.services["a"].traces(), 2);
        assert_eq!(a.total_traces(), 2);
    }

    #[test]
    fn trace_ids_compare_case_insensitively() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![span("AB"), span("ab")]))
            .unwrap();
        assert_eq!(a.total_traces(), 1);
    }

    #[test]
    fn a_trace_across_two_services_counts_once_in_total() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![span("01")])).unwrap();
        a.add_document(&doc(Some("b"), vec![span("01")])).unwrap();
        assert_eq!(a.services["a"].traces(), 1);
        assert_eq!(a.services["b"].traces(), 1);
        assert_eq!(a.total_traces(), 1);
    }

    #[test]
    fn spans_without_trace_id_are_not_traces() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![json!({"name": "x"})]))
            .unwrap();
        assert_eq!(a.total_spans(), 1);
        assert_eq!(a.total_traces(), 0);
    }

    #[test]
    fn document_without_resource_spans_is_an_error() {
        let mut a = Aggregate::default();
        assert!(a.add_document(&json!({"resourceMetrics": []})).is_err());
        assert!(a.add_document(&json!([])).is_err());
        assert_eq!(a.documents, 0);
    }

    #[test]
    fn empty_resource_spans_is_fine() {
        let mut a = Aggregate::default();
        a.add_document(&json!({"resourceSpans": []})).unwrap();
        assert_eq!(a.total_spans(), 0);
        assert_eq!(a.documents, 1);
    }

    #[test]
    fn snake_case_document_is_read() {
        let mut a = Aggregate::default();
        let d = json!({"resource_spans": [{"resource": {"attributes": [{"key": "service.name", "value": {"string_value": "s"}}]},
            "scope_spans": [{"spans": [{"trace_id": "01", "name": "x"}]}]}]});
        a.add_document(&d).unwrap();
        assert_eq!(a.services["s"].spans, 1);
    }

    #[test]
    fn legacy_instrumentation_library_spans_are_read() {
        let mut a = Aggregate::default();
        let d = json!({"resourceSpans": [{"resource": {"attributes": []},
            "instrumentationLibrarySpans": [{"instrumentationLibrary": {"name": "old"}, "spans": [{"name": "x"}]}]}]});
        a.add_document(&d).unwrap();
        assert_eq!(a.total_spans(), 1);
        assert!(a.total_bytes() > 0);
    }

    #[test]
    fn host_name_is_collected_and_preferred_over_host_id() {
        let mut a = Aggregate::default();
        let d = json!({"resourceSpans": [{"resource": {"attributes": [
            {"key": "host.id", "value": {"stringValue": "id-1"}},
            {"key": "host.name", "value": {"stringValue": "web-1"}}]}, "scopeSpans": []}]});
        a.add_document(&d).unwrap();
        assert_eq!(
            a.hosts.iter().cloned().collect::<Vec<_>>(),
            vec!["web-1".to_string()]
        );
    }

    #[test]
    fn host_id_is_used_without_host_name() {
        let mut a = Aggregate::default();
        let d = json!({"resourceSpans": [{"resource": {"attributes": [
            {"key": "host.id", "value": {"stringValue": "id-1"}}]}, "scopeSpans": []}]});
        a.add_document(&d).unwrap();
        assert!(a.hosts.contains("id-1"));
    }

    #[test]
    fn same_host_twice_counts_once() {
        let mut a = Aggregate::default();
        let d = json!({"resourceSpans": [
            {"resource": {"attributes": [{"key": "host.name", "value": {"stringValue": "h"}}]}, "scopeSpans": []},
            {"resource": {"attributes": [{"key": "host.name", "value": {"stringValue": "h"}}]}, "scopeSpans": []}]});
        a.add_document(&d).unwrap();
        assert_eq!(a.hosts.len(), 1);
    }

    #[test]
    fn resource_header_is_counted_once_per_resource() {
        let mut one = Aggregate::default();
        one.add_document(&doc(Some("a"), vec![])).unwrap();
        assert!(one.total_bytes() > 0);
        assert_eq!(one.total_spans(), 0);
    }

    #[test]
    fn bytes_grow_with_each_span() {
        let mut a = Aggregate::default();
        a.add_document(&doc(Some("a"), vec![span("01")])).unwrap();
        let one = a.total_bytes();
        a.add_document(&doc(Some("a"), vec![span("01")])).unwrap();
        assert!(a.total_bytes() > one);
    }

    #[test]
    fn attr_string_ignores_non_string_values() {
        let r = json!({"attributes": [{"key": "service.name", "value": {"intValue": "5"}}]});
        assert_eq!(attr_string(&r, "service.name"), None);
        assert_eq!(attr_string(&json!({}), "x"), None);
    }
}
