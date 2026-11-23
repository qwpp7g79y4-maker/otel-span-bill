//! The size estimate is checked against a real protobuf encoder: the byte
//! count below was produced by Python `google.protobuf` (3.12) with a
//! hand-written copy of the `opentelemetry.proto.trace.v1` message layout
//! (`Span`, `Event`, `Link`, `Status`, `KeyValue`, `AnyValue`), encoding the
//! same span as `tests/fixtures/rich-span.json` inside a `ScopeSpans`.

use otel_span_bill::size::span_len;
use serde_json::Value;

fn rich_span() -> Value {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rich-span.json"
    ))
    .unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    doc["resourceSpans"][0]["scopeSpans"][0]["spans"][0].clone()
}

#[test]
fn rich_span_matches_the_protobuf_encoder() {
    assert_eq!(span_len(&rich_span()), 370);
}

#[test]
fn empty_span_matches_the_protobuf_encoder() {
    assert_eq!(span_len(&serde_json::json!({})), 2);
}

#[test]
fn rich_span_without_links_is_smaller_by_the_link_size() {
    let mut s = rich_span();
    s.as_object_mut().unwrap().remove("links");
    assert!(span_len(&s) < 370);
}

#[test]
fn resource_and_scope_header_match_the_protobuf_encoder() {
    use otel_span_bill::size::block_header_len;
    let resource = serde_json::json!({
        "attributes": [
            {"key": "service.name", "value": {"stringValue": "checkout"}},
            {"key": "host.name", "value": {"stringValue": "web-1"}}],
        "droppedAttributesCount": 1});
    let scope = serde_json::json!({
        "name": "my.lib", "version": "1.2.3", "droppedAttributesCount": 2,
        "attributes": [{"key": "k", "value": {"stringValue": "v"}}]});
    assert_eq!(block_header_len(Some(&resource), None, 0), 54);
    assert_eq!(block_header_len(None, Some(&scope), 0), 29);
    assert_eq!(block_header_len(Some(&resource), Some(&scope), 0), 83);
}

#[test]
fn span_flags_take_a_two_byte_tag() {
    let with = serde_json::json!({"flags": 256});
    // 2-byte tag + 4 bytes = 6, plus the 2-byte span framing
    assert_eq!(span_len(&with), 8);
    assert_eq!(span_len(&serde_json::json!({"flags": 0})), 2);
}
