use lsys_core::api_utils::*;
use lsys_core::db::*;
use serde::Serialize;
use serde_json::{Value, json};

// ─── JsonData ────────────────────────────────────────────────────────────────

#[test]
fn json_data_default_values() {
    let data = JsonData::default();
    assert_eq!(data.code, "200");
    assert_eq!(data.sub_code, "ok");
    assert!(data.body.is_none());
}

#[test]
fn json_data_error_values() {
    let data = JsonData::error();
    assert_eq!(data.code, "500");
    assert_eq!(data.sub_code, "system");
    assert!(data.body.is_none());
}

#[test]
fn json_data_body_string() {
    let data = JsonData::body("hello");
    assert_eq!(data.body, Some(json!("hello")));
    assert_eq!(data.code, "200");
}

#[test]
fn json_data_body_number() {
    let data = JsonData::body(42);
    assert_eq!(data.body, Some(json!(42)));
}

#[test]
fn json_data_body_float() {
    let data = JsonData::body(3.1);
    assert_eq!(data.body, Some(json!(3.1)));
}

#[test]
fn json_data_body_bool() {
    let data = JsonData::body(true);
    assert_eq!(data.body, Some(json!(true)));
}

#[test]
fn json_data_body_array() {
    let data = JsonData::body(vec![1, 2, 3]);
    assert_eq!(data.body, Some(json!([1, 2, 3])));
}

#[test]
fn json_data_body_string_array() {
    let data = JsonData::body(vec!["a", "b"]);
    assert_eq!(data.body, Some(json!(["a", "b"])));
}

#[test]
fn json_data_body_object() {
    let data = JsonData::body(json!({"key": "value", "num": 1}));
    let body = data.body.unwrap();
    assert_eq!(body["key"], "value");
    assert_eq!(body["num"], 1);
}

#[test]
fn json_data_body_option_some() {
    let data = JsonData::body(Some(99));
    assert_eq!(data.body, Some(json!(99)));
}

#[test]
fn json_data_body_option_none() {
    let data = JsonData::body(Option::<i32>::None);
    assert_eq!(data.body, Some(Value::Null));
}

#[test]
fn json_data_body_null_value() {
    let data = JsonData::body(Value::Null);
    assert_eq!(data.body, Some(Value::Null));
}

#[test]
fn json_data_chained_set_code_and_sub_code() {
    let data = JsonData::body("test")
        .set_code(404)
        .set_sub_code("not_found");
    assert_eq!(data.code, "404");
    assert_eq!(data.sub_code, "not_found");
    assert_eq!(data.body, Some(json!("test")));
}

#[test]
fn json_data_set_body_overwrites() {
    let data = JsonData::body("first").set_body("second");
    assert_eq!(data.body, Some(json!("second")));
}

#[test]
fn json_data_set_code_with_string() {
    let data = JsonData::default().set_code("403");
    assert_eq!(data.code, "403");
}

#[test]
fn json_data_serialization_with_body() {
    let data = JsonData::body("hello");
    let v: Value = serde_json::to_value(&data).unwrap();
    assert_eq!(v["code"], "200");
    assert_eq!(v["sub_code"], "ok");
    assert_eq!(v["body"], "hello");
}

#[test]
fn json_data_serialization_without_body() {
    let data = JsonData::default();
    let v: Value = serde_json::to_value(&data).unwrap();
    assert_eq!(v["code"], "200");
    assert!(v["body"].is_null());
}

// ─── JsonIntoBody ────────────────────────────────────────────────────────────

#[test]
fn json_into_body_custom_struct() {
    #[derive(Serialize)]
    struct User {
        name: String,
        age: u32,
    }
    let user = User {
        name: "Alice".into(),
        age: 30,
    };
    let body = user.into_body();
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["age"], 30);
}

// ─── JsonPageTotal ───────────────────────────────────────────────────────────

#[test]
fn json_page_total_from_u64() {
    let total: JsonPageTotal = 100u64.into();
    let v = serde_json::to_value(&total).unwrap();
    assert_eq!(v, json!(100));
}

#[test]
fn json_page_total_from_i64() {
    let total: JsonPageTotal = 50i64.into();
    let v = serde_json::to_value(&total).unwrap();
    assert_eq!(v, json!(50));
}

#[test]
fn json_page_total_from_usize() {
    let total: JsonPageTotal = 75usize.into();
    let v = serde_json::to_value(&total).unwrap();
    assert_eq!(v, json!(75));
}

#[test]
fn json_page_total_none_serializes_to_null() {
    let total = JsonPageTotal::None;
    let v = serde_json::to_value(&total).unwrap();
    assert!(v.is_null());
}

#[test]
fn json_page_total_from_option_none_u64() {
    let total: JsonPageTotal = Option::<u64>::None.into();
    let v = serde_json::to_value(&total).unwrap();
    assert!(v.is_null());
}

#[test]
fn json_page_total_from_option_some_u64() {
    let total: JsonPageTotal = Some(42u64).into();
    let v = serde_json::to_value(&total).unwrap();
    assert_eq!(v, json!(42));
}

#[test]
fn json_page_total_from_option_none_i64() {
    let total: JsonPageTotal = Option::<i64>::None.into();
    let v = serde_json::to_value(&total).unwrap();
    assert!(v.is_null());
}

#[test]
fn json_page_total_from_option_some_i64() {
    let total: JsonPageTotal = Some(-10i64).into();
    let v = serde_json::to_value(&total).unwrap();
    assert_eq!(v, json!(-10));
}

#[test]
fn json_page_total_from_option_none_usize() {
    let total: JsonPageTotal = Option::<usize>::None.into();
    let v = serde_json::to_value(&total).unwrap();
    assert!(v.is_null());
}

#[test]
fn json_page_total_complex() {
    let row_val = PageTotalRowValue {
        exact: Some(200),
        over: None,
    };
    let total: JsonPageTotal = row_val.into();
    let v = serde_json::to_value(&total).unwrap();
    assert_eq!(v["exact"], 200);
    assert!(v["over"].is_null());
}

#[test]
fn json_page_total_from_option_none_complex() {
    let total: JsonPageTotal = Option::<PageTotalRowValue>::None.into();
    let v = serde_json::to_value(&total).unwrap();
    assert!(v.is_null());
}

// ─── JsonPageData ────────────────────────────────────────────────────────────

#[test]
fn json_page_data_total_u64() {
    let page = JsonPageData::total(vec![1, 2, 3], 100u64);
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["data"], json!([1, 2, 3]));
    assert_eq!(v["total"], json!(100));
    assert!(v.get("cursor").is_none());
}

#[test]
fn json_page_data_total_i64() {
    let page = JsonPageData::total(vec!["a", "b"], 50i64);
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["data"], json!(["a", "b"]));
    assert_eq!(v["total"], json!(50));
}

#[test]
fn json_page_data_total_none_omits_total_field() {
    let page = JsonPageData::total(vec![1], JsonPageTotal::None);
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["data"], json!([1]));
    // total field should not be present when None
    assert!(v.get("total").is_none());
}

#[test]
fn json_page_data_null_data_omits_data_field() {
    let page = JsonPageData::total(Value::Null, 10u64);
    let v = serde_json::to_value(&page).unwrap();
    // data is null, so it should be omitted
    assert!(v.get("data").is_none());
    assert_eq!(v["total"], json!(10));
}

#[test]
fn json_page_data_with_cursor() {
    let cursor = PageCursorValue {
        next: Some(100),
        prev: Some(50),
    };
    let page = JsonPageData::cursor(vec![1, 2], cursor, 200u64);
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["data"], json!([1, 2]));
    assert_eq!(v["total"], json!(200));
    assert_eq!(v["cursor"]["next"], json!(100));
    assert_eq!(v["cursor"]["prev"], json!(50));
}

#[test]
fn json_page_data_cursor_with_none_values() {
    let cursor = PageCursorValue {
        next: None,
        prev: None,
    };
    let page = JsonPageData::cursor(vec!["x"], cursor, 5u64);
    let v = serde_json::to_value(&page).unwrap();
    assert!(v["cursor"]["next"].is_null());
    assert!(v["cursor"]["prev"].is_null());
}

#[test]
fn json_page_data_set_extra() {
    let page = JsonPageData::total(vec![1], 10u64)
        .set_extra("timestamp", 1234567890u64)
        .set_extra("label", "test");
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["timestamp"], json!(1234567890));
    assert_eq!(v["label"], json!("test"));
    assert_eq!(v["data"], json!([1]));
    assert_eq!(v["total"], json!(10));
}

#[test]
fn json_page_data_set_extra_null_value() {
    let page = JsonPageData::total(vec![1], 5u64).set_extra("empty", Value::Null);
    let v = serde_json::to_value(&page).unwrap();
    assert!(v["empty"].is_null());
}

#[test]
fn json_page_data_complex_total() {
    let total_row = PageTotalRowValue {
        exact: Some(500),
        over: None,
    };
    let page = JsonPageData::total(vec![1, 2], total_row);
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["total"]["exact"], json!(500));
    assert!(v["total"]["over"].is_null());
}

#[test]
fn json_page_data_over_total() {
    let total_row = PageTotalRowValue {
        exact: None,
        over: Some(10000),
    };
    let page = JsonPageData::total(vec![1], total_row);
    let v = serde_json::to_value(&page).unwrap();
    assert!(v["total"]["exact"].is_null());
    assert_eq!(v["total"]["over"], json!(10000));
}

#[test]
fn json_page_data_as_json_data_body() {
    let page = JsonPageData::total(vec![1, 2], 42u64);
    let data = JsonData::body(page);
    assert_eq!(data.code, "200");
    let body = data.body.unwrap();
    assert_eq!(body["data"], json!([1, 2]));
    assert_eq!(body["total"], json!(42));
}

#[test]
fn json_page_data_empty_data_array() {
    let page = JsonPageData::total(Vec::<i32>::new(), 0u64);
    let v = serde_json::to_value(&page).unwrap();
    assert_eq!(v["data"], json!([]));
    assert_eq!(v["total"], json!(0));
}

// ─── JsonResponse ────────────────────────────────────────────────────────────

#[test]
fn json_response_default_values() {
    let resp = JsonResponse::default();
    let v = resp.to_value();
    assert_eq!(v["result"]["code"], "200");
    assert_eq!(v["result"]["state"], "ok");
    assert_eq!(v["result"]["message"], "ok");
    assert!(v.get("response").is_none());
}

#[test]
fn json_response_with_data() {
    let data = JsonData::body("hello");
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["result"]["code"], "200");
    assert_eq!(v["result"]["state"], "ok");
    // string body passes through unchanged
    assert_eq!(v["response"], "hello");
}

#[test]
fn json_response_with_message() {
    let resp = JsonResponse::message("custom message");
    let v = resp.to_value();
    assert_eq!(v["result"]["message"], "custom message");
    assert_eq!(v["result"]["code"], "200");
}

#[test]
fn json_response_set_data_and_message() {
    let data = JsonData::body(42).set_code(201).set_sub_code("created");
    let resp = JsonResponse::default()
        .set_data(data)
        .set_message("resource created");
    let v = resp.to_value();
    assert_eq!(v["result"]["code"], "201");
    assert_eq!(v["result"]["state"], "created");
    assert_eq!(v["result"]["message"], "resource created");
    // numbers are converted to strings by convert_value
    assert_eq!(v["response"], "42");
}

#[test]
fn json_response_no_body_omits_response_key() {
    let resp = JsonResponse::data(JsonData::default());
    let v = resp.to_value();
    assert!(v.get("response").is_none());
}

#[test]
fn json_response_error_data() {
    let resp = JsonResponse::data(JsonData::error()).set_message("internal error");
    let v = resp.to_value();
    assert_eq!(v["result"]["code"], "500");
    assert_eq!(v["result"]["state"], "system");
    assert_eq!(v["result"]["message"], "internal error");
}

// ─── JsonResponse::convert_value ─────────────────────────────────────────────

#[test]
fn json_response_converts_number_to_string() {
    let data = JsonData::body(json!({"count": 123}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["response"]["count"], "123");
}

#[test]
fn json_response_converts_float_to_string() {
    let data = JsonData::body(json!({"price": 9.99}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["response"]["price"], "9.99");
}

#[test]
fn json_response_converts_bool_true_to_one() {
    let data = JsonData::body(json!({"active": true}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["response"]["active"], "1");
}

#[test]
fn json_response_converts_bool_false_to_zero() {
    let data = JsonData::body(json!({"active": false}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["response"]["active"], "0");
}

#[test]
fn json_response_preserves_strings() {
    let data = JsonData::body(json!({"name": "alice"}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["response"]["name"], "alice");
}

#[test]
fn json_response_preserves_null() {
    let data = JsonData::body(json!({"empty": null}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert!(v["response"]["empty"].is_null());
}

#[test]
fn json_response_converts_nested_object() {
    let data = JsonData::body(json!({
        "user": {"id": 1, "active": true, "name": "bob"}
    }));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    assert_eq!(v["response"]["user"]["id"], "1");
    assert_eq!(v["response"]["user"]["active"], "1");
    assert_eq!(v["response"]["user"]["name"], "bob");
}

#[test]
fn json_response_converts_array_elements() {
    let data = JsonData::body(json!([1, true, "hello", null]));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    let arr = v["response"].as_array().unwrap();
    assert_eq!(arr[0], "1");
    assert_eq!(arr[1], "1");
    assert_eq!(arr[2], "hello");
    assert!(arr[3].is_null());
}

#[test]
fn json_response_converts_nested_array_in_object() {
    let data = JsonData::body(json!({"items": [10, 20, false]}));
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    let items = v["response"]["items"].as_array().unwrap();
    assert_eq!(items[0], "10");
    assert_eq!(items[1], "20");
    assert_eq!(items[2], "0");
}

#[test]
fn json_response_with_page_data() {
    let page = JsonPageData::total(vec![10, 20], 100u64);
    let data = JsonData::body(page);
    let resp = JsonResponse::data(data);
    let v = resp.to_value();
    // total is a number, converted to string
    assert_eq!(v["response"]["total"], "100");
    // array elements converted
    let arr = v["response"]["data"].as_array().unwrap();
    assert_eq!(arr[0], "10");
    assert_eq!(arr[1], "20");
}

// ─── compute_rest_sign ───────────────────────────────────────────────────────

#[test]
fn rest_sign_deterministic_no_optional_fields() {
    let data = RestSignData {
        client_id: "app1",
        version: "1.0",
        timestamp: "1700000000",
        request_ip: None,
        method: None,
        token: None,
        payload: None,
    };
    let result = compute_rest_sign(&data, "secret");
    // BTreeMap ordering: client_id, timestamp, version
    assert!(result.raw_string.contains("client_id=app1"));
    assert!(result.raw_string.contains("timestamp=1700000000"));
    assert!(result.raw_string.contains("version=1.0"));
    // verify determinism
    let result2 = compute_rest_sign(&data, "secret");
    assert_eq!(result.signature, result2.signature);
    assert_eq!(result.raw_string, result2.raw_string);
}

#[test]
fn rest_sign_with_all_fields() {
    let payload = json!({"key": "value"});
    let data = RestSignData {
        client_id: "app1",
        version: "1.0",
        timestamp: "1700000000",
        request_ip: Some("192.168.1.1"),
        method: Some("POST"),
        token: Some("tok123"),
        payload: Some(&payload),
    };
    let result = compute_rest_sign(&data, "my_key");
    // All optional fields should be in the raw string
    assert!(result.raw_string.contains("request_ip=192.168.1.1"));
    assert!(result.raw_string.contains("method=POST"));
    assert!(result.raw_string.contains("token=tok123"));
    // payload JSON should be appended
    assert!(result.raw_string.contains(r#""key":"value""#));
    assert!(!result.signature.is_empty());
    assert_eq!(result.signature.len(), 32); // MD5 hex = 32 chars
}

#[test]
fn rest_sign_md5_verification() {
    // Manually compute the expected MD5 for a known input
    let data = RestSignData {
        client_id: "c1",
        version: "v1",
        timestamp: "100",
        request_ip: None,
        method: None,
        token: None,
        payload: None,
    };
    let result = compute_rest_sign(&data, "key");
    // Reconstruct: BTreeMap keys sorted => client_id, timestamp, version
    let expected_raw = "client_id=c1&timestamp=100&version=v1";
    assert_eq!(result.raw_string, expected_raw);
    let hash_input = format!("{}key", expected_raw);
    let expected_sig = format!("{:x}", md5::compute(hash_input.as_bytes()));
    assert_eq!(result.signature, expected_sig);
}

#[test]
fn rest_sign_with_payload_md5_verification() {
    let payload = json!({"a": 1});
    let data = RestSignData {
        client_id: "c1",
        version: "v1",
        timestamp: "100",
        request_ip: None,
        method: None,
        token: None,
        payload: Some(&payload),
    };
    let result = compute_rest_sign(&data, "key");
    let payload_str = serde_json::to_string(&payload).unwrap();
    let expected_raw = format!("client_id=c1&timestamp=100&version=v1{}", payload_str);
    assert_eq!(result.raw_string, expected_raw);
    let hash_input = format!("{}key", expected_raw);
    let expected_sig = format!("{:x}", md5::compute(hash_input.as_bytes()));
    assert_eq!(result.signature, expected_sig);
}

#[test]
fn rest_sign_different_keys_produce_different_signatures() {
    let data = RestSignData {
        client_id: "app1",
        version: "1.0",
        timestamp: "100",
        request_ip: None,
        method: None,
        token: None,
        payload: None,
    };
    let r1 = compute_rest_sign(&data, "key_a");
    let r2 = compute_rest_sign(&data, "key_b");
    assert_eq!(r1.raw_string, r2.raw_string);
    assert_ne!(r1.signature, r2.signature);
}

#[test]
fn rest_sign_empty_payload_object() {
    let payload = json!({});
    let data = RestSignData {
        client_id: "c1",
        version: "v1",
        timestamp: "100",
        request_ip: None,
        method: None,
        token: None,
        payload: Some(&payload),
    };
    let result = compute_rest_sign(&data, "key");
    // empty object should still be appended as "{}"
    assert!(result.raw_string.ends_with("{}"));
}

#[test]
fn rest_sign_special_characters_url_encoded() {
    let data = RestSignData {
        client_id: "app&id=1",
        version: "1.0",
        timestamp: "100",
        request_ip: None,
        method: None,
        token: None,
        payload: None,
    };
    let result = compute_rest_sign(&data, "key");
    // & and = should be URL-encoded in client_id
    assert!(result.raw_string.contains("client_id=app%26id%3D1"));
}

// ─── compute_service_sign ────────────────────────────────────────────────────

#[test]
fn service_sign_with_explicit_timestamp() {
    let result = compute_service_sign("my_api_key", Some("1700000000"));
    assert_eq!(result.timestamp, "1700000000");
    assert_eq!(result.raw_string, "my_api_key1700000000");
    let expected = format!("{:x}", md5::compute(b"my_api_key1700000000"));
    assert_eq!(result.signature, expected);
}

#[test]
fn service_sign_auto_timestamp() {
    let result = compute_service_sign("secret", None);
    // timestamp should be a valid number
    let ts: u64 = result
        .timestamp
        .parse()
        .expect("timestamp should be numeric");
    assert!(ts > 0);
    assert_eq!(result.raw_string, format!("secret{}", result.timestamp));
    assert_eq!(result.signature.len(), 32);
}

#[test]
fn service_sign_deterministic() {
    let r1 = compute_service_sign("key", Some("999"));
    let r2 = compute_service_sign("key", Some("999"));
    assert_eq!(r1.signature, r2.signature);
    assert_eq!(r1.raw_string, r2.raw_string);
}

#[test]
fn service_sign_different_keys_differ() {
    let r1 = compute_service_sign("key_a", Some("100"));
    let r2 = compute_service_sign("key_b", Some("100"));
    assert_ne!(r1.signature, r2.signature);
}

#[test]
fn service_sign_different_timestamps_differ() {
    let r1 = compute_service_sign("key", Some("100"));
    let r2 = compute_service_sign("key", Some("200"));
    assert_ne!(r1.signature, r2.signature);
}

#[test]
fn service_sign_empty_key() {
    let result = compute_service_sign("", Some("100"));
    assert_eq!(result.raw_string, "100");
    let expected = format!("{:x}", md5::compute(b"100"));
    assert_eq!(result.signature, expected);
}

#[test]
fn service_sign_header_constants() {
    assert_eq!(SERVICE_TIMESTAMP_HEADER, "X-Timestamp");
    assert_eq!(SERVICE_SIGNATURE_HEADER, "X-Signature");
}

// ─── PageTotalRowValue ───────────────────────────────────────────────────────

#[test]
fn page_total_row_value_from_exact() {
    let total = TotalRow::Exact(42);
    let val: PageTotalRowValue = total.into();
    assert_eq!(val.exact, Some(42));
    assert_eq!(val.over, None);
}

#[test]
fn page_total_row_value_from_over() {
    let total = TotalRow::Over(10000);
    let val: PageTotalRowValue = total.into();
    assert_eq!(val.exact, None);
    assert_eq!(val.over, Some(10000));
}

#[test]
fn page_total_row_value_from_ref_exact() {
    let total = TotalRow::Exact(99);
    let val: PageTotalRowValue = (&total).into();
    assert_eq!(val.exact, Some(99));
    assert_eq!(val.over, None);
}

#[test]
fn page_total_row_value_from_ref_over() {
    let total = TotalRow::Over(5000);
    let val: PageTotalRowValue = (&total).into();
    assert_eq!(val.exact, None);
    assert_eq!(val.over, Some(5000));
}

#[test]
fn page_total_row_value_serialization_exact() {
    let val = PageTotalRowValue {
        exact: Some(100),
        over: None,
    };
    let v = serde_json::to_value(&val).unwrap();
    assert_eq!(v, json!({"exact": 100, "over": null}));
}

#[test]
fn page_total_row_value_serialization_over() {
    let val = PageTotalRowValue {
        exact: None,
        over: Some(10000),
    };
    let v = serde_json::to_value(&val).unwrap();
    assert_eq!(v, json!({"exact": null, "over": 10000}));
}

#[test]
fn page_total_row_value_serialization_both() {
    let val = PageTotalRowValue {
        exact: Some(50),
        over: Some(100),
    };
    let v = serde_json::to_value(&val).unwrap();
    assert_eq!(v, json!({"exact": 50, "over": 100}));
}

// ─── PageCursorValue ─────────────────────────────────────────────────────────

#[test]
fn page_cursor_value_from_cursor_page_data() {
    let cpd = CursorPageData {
        next_cursor: Some(100u64),
        prev_cursor: Some(50u64),
    };
    let val: PageCursorValue = cpd.into();
    assert_eq!(val.next, Some(100));
    assert_eq!(val.prev, Some(50));
}

#[test]
fn page_cursor_value_from_cursor_page_data_none() {
    let cpd: CursorPageData<u64> = CursorPageData {
        next_cursor: None,
        prev_cursor: None,
    };
    let val: PageCursorValue = cpd.into();
    assert_eq!(val.next, None);
    assert_eq!(val.prev, None);
}

#[test]
fn page_cursor_value_from_ref() {
    let cpd = CursorPageData {
        next_cursor: Some(200u64),
        prev_cursor: None,
    };
    let val: PageCursorValue = (&cpd).into();
    assert_eq!(val.next, Some(200));
    assert_eq!(val.prev, None);
}

#[test]
fn page_cursor_value_serialization() {
    let val = PageCursorValue {
        next: Some(10),
        prev: Some(5),
    };
    let v = serde_json::to_value(&val).unwrap();
    assert_eq!(v, json!({"next": 10, "prev": 5}));
}

#[test]
fn page_cursor_value_serialization_nulls() {
    let val = PageCursorValue {
        next: None,
        prev: None,
    };
    let v = serde_json::to_value(&val).unwrap();
    assert_eq!(v, json!({"next": null, "prev": null}));
}

// ─── Integration: full pipeline ──────────────────────────────────────────────

#[test]
fn full_pipeline_page_data_to_response() {
    let cursor = PageCursorValue {
        next: Some(999),
        prev: Some(1),
    };
    let total = PageTotalRowValue {
        exact: Some(500),
        over: None,
    };
    let page = JsonPageData::cursor(vec![10, 20, 30], cursor, total).set_extra("page_size", 3);
    let data = JsonData::body(page).set_code(200);
    let resp = JsonResponse::data(data).set_message("success");
    let v = resp.to_value();

    // result block
    assert_eq!(v["result"]["code"], "200");
    assert_eq!(v["result"]["state"], "ok");
    assert_eq!(v["result"]["message"], "success");

    // response block (all numbers converted to strings)
    assert_eq!(v["response"]["total"]["exact"], "500");
    assert!(v["response"]["total"]["over"].is_null());
    assert_eq!(v["response"]["cursor"]["next"], "999");
    assert_eq!(v["response"]["cursor"]["prev"], "1");
    assert_eq!(v["response"]["page_size"], "3");

    let data_arr = v["response"]["data"].as_array().unwrap();
    assert_eq!(data_arr.len(), 3);
    assert_eq!(data_arr[0], "10");
}

#[test]
fn full_pipeline_total_row_through_finalize() {
    let param = TotalParam::Threshold(100);
    let query = param.total_count_query();
    let total = query.finalize(50);
    assert!(total.is_exact());
    let val: PageTotalRowValue = total.into();
    assert_eq!(val.exact, Some(50));
    assert_eq!(val.over, None);

    let param2 = TotalParam::Threshold(100);
    let query2 = param2.total_count_query();
    let total2 = query2.finalize(200);
    assert!(!total2.is_exact());
    let val2: PageTotalRowValue = total2.into();
    assert_eq!(val2.exact, None);
    assert_eq!(val2.over, Some(100));
}
