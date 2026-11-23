//! Estimate the OTLP protobuf wire size of a span from its OTLP-JSON form.
//!
//! Neither Datadog nor Grafana documents which byte count they bill, so
//! this crate uses the size of the span as an `opentelemetry.proto.trace.v1.Span`
//! message in protobuf encoding, plus 1 tag byte and the length prefix of
//! the `ScopeSpans.spans` field it sits in. Default (zero) fields are not
//! counted, as proto3 omits them.

use serde_json::Value;

/// Length of a protobuf varint.
pub fn varint_len(mut v: u64) -> u64 {
    let mut n = 1;
    while v >= 0x80 {
        v >>= 7;
        n += 1;
    }
    n
}

/// A length-delimited field: tag (field numbers under 16) + length + body.
fn ld(len: u64) -> u64 {
    1 + varint_len(len) + len
}

/// Read a JSON field by its camelCase or snake_case name.
pub fn field<'a>(v: &'a Value, camel: &str, snake: &str) -> Option<&'a Value> {
    v.get(camel).or_else(|| v.get(snake))
}

fn str_field_len(v: Option<&Value>) -> u64 {
    match v.and_then(Value::as_str) {
        Some(s) if !s.is_empty() => ld(s.len() as u64),
        _ => 0,
    }
}

/// Hex ids (trace id, span id) are 2 hex characters per byte.
fn id_field_len(v: Option<&Value>) -> u64 {
    match v.and_then(Value::as_str) {
        Some(s) if !s.is_empty() => ld(s.len().div_ceil(2) as u64),
        _ => 0,
    }
}

/// An unsigned 64-bit value that OTLP-JSON writes as a string or a number.
fn as_u64(v: &Value) -> Option<u64> {
    match v {
        Value::String(s) => s.trim().parse().ok(),
        Value::Number(n) => n.as_u64(),
        _ => None,
    }
}

fn as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::String(s) => s.trim().parse().ok(),
        Value::Number(n) => n.as_i64(),
        _ => None,
    }
}

fn is_nonzero(v: &Value) -> bool {
    match as_u64(v) {
        Some(n) => n != 0,
        None => !matches!(v, Value::String(s) if s.is_empty()) && !v.is_null(),
    }
}

/// `fixed64` field: 1 tag byte + 8 bytes when non-zero.
fn fixed64_field_len(v: Option<&Value>) -> u64 {
    match v {
        Some(x) if is_nonzero(x) => 9,
        _ => 0,
    }
}

/// `fixed32` field: 1 tag byte + 4 bytes when non-zero.
fn fixed32_field_len(v: Option<&Value>) -> u64 {
    match v.and_then(as_u64) {
        Some(n) if n != 0 => 5,
        _ => 0,
    }
}

/// `uint32` varint field when non-zero.
fn count_field_len(v: Option<&Value>) -> u64 {
    match v.and_then(as_u64) {
        Some(n) if n != 0 => 1 + varint_len(n),
        _ => 0,
    }
}

pub fn span_kind_number(v: &Value) -> u64 {
    match v {
        Value::String(s) => match s.as_str() {
            "SPAN_KIND_INTERNAL" => 1,
            "SPAN_KIND_SERVER" => 2,
            "SPAN_KIND_CLIENT" => 3,
            "SPAN_KIND_PRODUCER" => 4,
            "SPAN_KIND_CONSUMER" => 5,
            other => other.parse().unwrap_or(0),
        },
        Value::Number(n) => n.as_u64().unwrap_or(0),
        _ => 0,
    }
}

pub fn status_code_number(v: &Value) -> u64 {
    match v {
        Value::String(s) => match s.as_str() {
            "STATUS_CODE_OK" => 1,
            "STATUS_CODE_ERROR" => 2,
            other => other.parse().unwrap_or(0),
        },
        Value::Number(n) => n.as_u64().unwrap_or(0),
        _ => 0,
    }
}

/// Decoded length of a base64 string (OTLP-JSON writes `bytesValue` as base64).
fn base64_decoded_len(s: &str) -> u64 {
    let s = s.trim_end_matches('=');
    (s.len() as u64) * 3 / 4
}

/// Body length of an `AnyValue` message.
fn any_value_body(v: &Value) -> u64 {
    if let Some(s) = field(v, "stringValue", "string_value").and_then(Value::as_str) {
        // A oneof member is written even when empty.
        return ld(s.len() as u64);
    }
    if field(v, "boolValue", "bool_value").is_some() {
        return 2;
    }
    if let Some(i) = field(v, "intValue", "int_value") {
        let n = as_i64(i).unwrap_or(0);
        // Negative int64 values take 10 bytes as a varint.
        let len = if n < 0 { 10 } else { varint_len(n as u64) };
        return 1 + len;
    }
    if field(v, "doubleValue", "double_value").is_some() {
        return 9;
    }
    if let Some(arr) = field(v, "arrayValue", "array_value") {
        let items = arr.get("values").and_then(Value::as_array);
        let body: u64 = items
            .map(|a| a.iter().map(|x| ld(any_value_body(x))).sum())
            .unwrap_or(0);
        return ld(body);
    }
    if let Some(kv) = field(v, "kvlistValue", "kvlist_value") {
        let body = key_values_len(kv.get("values"), 1);
        return ld(body);
    }
    if let Some(b) = field(v, "bytesValue", "bytes_value").and_then(Value::as_str) {
        return ld(base64_decoded_len(b));
    }
    0
}

/// Total length of a repeated `KeyValue` field. `tag_len` is 1 for field
/// numbers under 16 (all fields in OTLP traces).
fn key_values_len(list: Option<&Value>, tag_len: u64) -> u64 {
    let Some(items) = list.and_then(Value::as_array) else {
        return 0;
    };
    items
        .iter()
        .map(|kv| {
            let key = kv.get("key").and_then(Value::as_str).unwrap_or("");
            let key_len = if key.is_empty() {
                0
            } else {
                ld(key.len() as u64)
            };
            let value_len = kv.get("value").map(|v| ld(any_value_body(v))).unwrap_or(0);
            let body = key_len + value_len;
            tag_len + varint_len(body) + body
        })
        .sum()
}

/// Length of the `attributes` of an object, as a repeated KeyValue field.
pub fn attributes_len(obj: &Value) -> u64 {
    key_values_len(obj.get("attributes"), 1)
}

fn event_len(e: &Value) -> u64 {
    let body = fixed64_field_len(field(e, "timeUnixNano", "time_unix_nano"))
        + str_field_len(e.get("name"))
        + attributes_len(e)
        + count_field_len(field(
            e,
            "droppedAttributesCount",
            "dropped_attributes_count",
        ));
    ld(body)
}

fn link_len(l: &Value) -> u64 {
    let body = id_field_len(field(l, "traceId", "trace_id"))
        + id_field_len(field(l, "spanId", "span_id"))
        + str_field_len(field(l, "traceState", "trace_state"))
        + attributes_len(l)
        + count_field_len(field(
            l,
            "droppedAttributesCount",
            "dropped_attributes_count",
        ))
        + fixed32_field_len(l.get("flags"));
    ld(body)
}

fn status_len(s: &Value) -> u64 {
    let msg = str_field_len(s.get("message"));
    let code = match s.get("code").map(status_code_number) {
        Some(c) if c != 0 => 1 + varint_len(c),
        _ => 0,
    };
    ld(msg + code)
}

/// Estimated size of one span inside its `ScopeSpans.spans` field.
pub fn span_len(span: &Value) -> u64 {
    let mut body = id_field_len(field(span, "traceId", "trace_id"))
        + id_field_len(field(span, "spanId", "span_id"))
        + str_field_len(field(span, "traceState", "trace_state"))
        + id_field_len(field(span, "parentSpanId", "parent_span_id"))
        + str_field_len(span.get("name"));
    if let Some(k) = span.get("kind").map(span_kind_number) {
        if k != 0 {
            body += 1 + varint_len(k);
        }
    }
    body += fixed64_field_len(field(span, "startTimeUnixNano", "start_time_unix_nano"));
    body += fixed64_field_len(field(span, "endTimeUnixNano", "end_time_unix_nano"));
    body += attributes_len(span);
    body += count_field_len(field(
        span,
        "droppedAttributesCount",
        "dropped_attributes_count",
    ));
    if let Some(events) = span.get("events").and_then(Value::as_array) {
        body += events.iter().map(event_len).sum::<u64>();
    }
    body += count_field_len(field(span, "droppedEventsCount", "dropped_events_count"));
    if let Some(links) = span.get("links").and_then(Value::as_array) {
        body += links.iter().map(link_len).sum::<u64>();
    }
    body += count_field_len(field(span, "droppedLinksCount", "dropped_links_count"));
    if let Some(status) = span.get("status") {
        body += status_len(status);
    }
    // Field number 16 needs a 2-byte tag, so span flags take 6 bytes.
    if fixed32_field_len(span.get("flags")) > 0 {
        body += 6;
    }
    ld(body)
}

/// Size of the `Resource` and `InstrumentationScope` headers of a block,
/// including their field framing, plus `schema_url` fields.
pub fn block_header_len(resource: Option<&Value>, scope: Option<&Value>, schema_urls: u64) -> u64 {
    let mut n = schema_urls;
    if let Some(r) = resource {
        let body = attributes_len(r)
            + count_field_len(field(
                r,
                "droppedAttributesCount",
                "dropped_attributes_count",
            ));
        n += ld(body);
    }
    if let Some(s) = scope {
        let body = str_field_len(s.get("name"))
            + str_field_len(s.get("version"))
            + attributes_len(s)
            + count_field_len(field(
                s,
                "droppedAttributesCount",
                "dropped_attributes_count",
            ));
        n += ld(body);
    }
    n
}

/// Length of a `schema_url` string field.
pub fn schema_url_len(v: Option<&Value>) -> u64 {
    str_field_len(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn varint_lengths() {
        assert_eq!(varint_len(0), 1);
        assert_eq!(varint_len(127), 1);
        assert_eq!(varint_len(128), 2);
        assert_eq!(varint_len(16_383), 2);
        assert_eq!(varint_len(16_384), 3);
        assert_eq!(varint_len(u64::MAX), 10);
    }

    #[test]
    fn empty_span_is_two_bytes() {
        // tag + zero length
        assert_eq!(span_len(&json!({})), 2);
    }

    #[test]
    fn ids_are_half_their_hex_length() {
        let s =
            json!({"traceId": "0102030405060708090a0b0c0d0e0f10", "spanId": "0102030405060708"});
        // trace id: 1+1+16 = 18, span id: 1+1+8 = 10, body 28 -> 1+1+28
        assert_eq!(span_len(&s), 30);
    }

    #[test]
    fn name_counts_its_utf8_bytes() {
        let s = json!({"name": "é"});
        // name field: 1+1+2 = 4, framing 2
        assert_eq!(span_len(&s), 6);
    }

    #[test]
    fn zero_timestamps_are_omitted() {
        let zero = json!({"startTimeUnixNano": "0", "endTimeUnixNano": 0});
        assert_eq!(span_len(&zero), 2);
        let set = json!({"startTimeUnixNano": "1700000000000000000", "endTimeUnixNano": "1700000000000000001"});
        // 9 + 9 = 18 body -> 20
        assert_eq!(span_len(&set), 20);
    }

    #[test]
    fn kind_names_and_numbers_agree() {
        assert_eq!(
            span_len(&json!({"kind": "SPAN_KIND_SERVER"})),
            span_len(&json!({"kind": 2}))
        );
        assert_eq!(span_len(&json!({"kind": 0})), 2);
        assert_eq!(span_len(&json!({"kind": 2})), 4);
    }

    #[test]
    fn snake_case_names_are_accepted() {
        let camel = json!({"traceId": "00112233445566778899aabbccddeeff"});
        let snake = json!({"trace_id": "00112233445566778899aabbccddeeff"});
        assert_eq!(span_len(&camel), span_len(&snake));
    }

    #[test]
    fn string_attribute_size() {
        let s = json!({"attributes": [{"key": "k", "value": {"stringValue": "vv"}}]});
        // key: 1+1+1 = 3; value: AnyValue body = 1+1+2 = 4, wrapped 1+1+4 = 6;
        // KeyValue body 9, wrapped 1+1+9 = 11; span body 11 -> 13
        assert_eq!(span_len(&s), 13);
    }

    #[test]
    fn int_attribute_negative_takes_ten_bytes() {
        let pos = json!({"attributes": [{"key": "k", "value": {"intValue": "5"}}]});
        let neg = json!({"attributes": [{"key": "k", "value": {"intValue": "-5"}}]});
        assert_eq!(span_len(&neg) - span_len(&pos), 9);
    }

    #[test]
    fn bool_and_double_attribute_sizes() {
        let b = json!({"attributes": [{"key": "k", "value": {"boolValue": false}}]});
        let d = json!({"attributes": [{"key": "k", "value": {"doubleValue": 1.5}}]});
        assert_eq!(span_len(&d) - span_len(&b), 7);
    }

    #[test]
    fn array_attribute_counts_elements() {
        let one = json!({"attributes": [{"key": "k", "value": {"arrayValue": {"values": [{"stringValue": "a"}]}}}]});
        let two = json!({"attributes": [{"key": "k", "value": {"arrayValue": {"values": [{"stringValue": "a"}, {"stringValue": "a"}]}}}]});
        // one more AnyValue: tag + len + (tag + len + 'a') = 5
        assert_eq!(span_len(&two) - span_len(&one), 5);
    }

    #[test]
    fn kvlist_attribute_counts_inner_pairs() {
        let v = json!({"attributes": [{"key": "k", "value": {"kvlistValue": {"values": [{"key": "a", "value": {"stringValue": "b"}}]}}}]});
        let empty = json!({"attributes": [{"key": "k", "value": {"kvlistValue": {"values": []}}}]});
        assert!(span_len(&v) > span_len(&empty));
    }

    #[test]
    fn bytes_value_uses_decoded_length() {
        let a = json!({"attributes": [{"key": "k", "value": {"bytesValue": "AAAA"}}]});
        let b = json!({"attributes": [{"key": "k", "value": {"bytesValue": "AAAAAAAA"}}]});
        assert_eq!(span_len(&b) - span_len(&a), 3);
    }

    #[test]
    fn events_and_links_add_size() {
        let base = json!({"name": "x"});
        let ev = json!({"name": "x", "events": [{"name": "e", "timeUnixNano": "5"}]});
        let ln = json!({"name": "x", "links": [{"traceId": "00112233445566778899aabbccddeeff", "spanId": "0011223344556677"}]});
        assert!(span_len(&ev) > span_len(&base));
        assert!(span_len(&ln) > span_len(&base));
    }

    #[test]
    fn status_with_message_and_code() {
        let none = json!({});
        let unset = json!({"status": {}});
        let err = json!({"status": {"code": "STATUS_CODE_ERROR", "message": "boom"}});
        assert_eq!(span_len(&unset) - span_len(&none), 2);
        // message 1+1+4 = 6, code 2 -> status body 8 -> 10
        assert_eq!(span_len(&err) - span_len(&none), 10);
    }

    #[test]
    fn dropped_counts_are_counted_when_non_zero() {
        let z = json!({"droppedAttributesCount": 0});
        let n = json!({"droppedAttributesCount": 3});
        assert_eq!(span_len(&n) - span_len(&z), 2);
    }

    #[test]
    fn bigger_span_never_smaller() {
        let small = json!({"name": "a"});
        let big = json!({"name": "aaaaaaaaaa"});
        assert!(span_len(&big) > span_len(&small));
    }

    #[test]
    fn long_name_grows_the_length_prefix() {
        let n127 = json!({"name": "a".repeat(127)});
        let n128 = json!({"name": "a".repeat(128)});
        // 128 chars needs a 2-byte length at both the name and the span level
        assert!(span_len(&n128) - span_len(&n127) >= 2);
    }

    #[test]
    fn block_header_counts_resource_and_scope() {
        let res = json!({"attributes": [{"key": "service.name", "value": {"stringValue": "a"}}]});
        let scope = json!({"name": "lib", "version": "1"});
        assert!(block_header_len(Some(&res), None, 0) > 0);
        assert!(
            block_header_len(Some(&res), Some(&scope), 0) > block_header_len(Some(&res), None, 0)
        );
        assert_eq!(block_header_len(None, None, 0), 0);
        assert_eq!(block_header_len(None, None, 7), 7);
    }

    #[test]
    fn status_and_kind_numbers() {
        assert_eq!(status_code_number(&json!("STATUS_CODE_ERROR")), 2);
        assert_eq!(status_code_number(&json!(1)), 1);
        assert_eq!(status_code_number(&json!("x")), 0);
        assert_eq!(span_kind_number(&json!("SPAN_KIND_CONSUMER")), 5);
        assert_eq!(span_kind_number(&json!("3")), 3);
    }
}
