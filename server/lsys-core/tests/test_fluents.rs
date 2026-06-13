use lsys_core::fluent_message;
use lsys_core::fluents::{FluentData, FluentMgr};
use std::fs;
use std::path::Path;

const TEST_LOCALE_DIR: &str = "/tmp/lsys_test_locale";

fn setup_locale_dir() {
    let dir = Path::new(TEST_LOCALE_DIR);
    if dir.exists() {
        fs::remove_dir_all(dir).unwrap();
    }

    // en_US locale
    let en_dir = dir.join("en_US");
    fs::create_dir_all(&en_dir).unwrap();
    fs::write(
        en_dir.join("lsys-core.ftl"),
        "\
hello = Hello World
greeting = Hello, { $name }!
error-msg = Error: { $msg }
welcome = Welcome, { $name }! You have { $count } messages.
",
    )
    .unwrap();

    // zh_CN locale (deliberately missing "welcome" for fallback testing)
    let zh_dir = dir.join("zh_CN");
    fs::create_dir_all(&zh_dir).unwrap();
    fs::write(
        zh_dir.join("lsys-core.ftl"),
        "\
hello = 你好世界
greeting = 你好, { $name }!
error-msg = 错误: { $msg }
",
    )
    .unwrap();
}

fn cleanup_locale_dir() {
    let _ = fs::remove_dir_all(TEST_LOCALE_DIR);
}

// ─── FluentMgr creation ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_fluent_mgr_creation_with_explicit_list() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"])).await;
    assert!(mgr.is_ok(), "FluentMgr::new should succeed");
    cleanup_locale_dir();
}

#[tokio::test]
async fn test_fluent_mgr_auto_discovery() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", None).await.unwrap();

    let bundle = mgr.locale(Some("en_US"));
    let msg = fluent_message!("hello");
    let result = bundle.format_message(&msg);
    assert_eq!(result, "Hello World");
    cleanup_locale_dir();
}

// ─── format_message ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fluent_format_simple_message() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("en_US"));
    let msg = fluent_message!("hello");
    assert_eq!(bundle.format_message(&msg), "Hello World");
    cleanup_locale_dir();
}

#[tokio::test]
async fn test_fluent_format_with_named_args() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("en_US"));
    let msg = fluent_message!("greeting", {"name": "Alice"});
    assert_eq!(bundle.format_message(&msg), "Hello, Alice!");
    cleanup_locale_dir();
}

#[tokio::test]
async fn test_fluent_format_with_multiple_args() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("en_US"));
    let msg = fluent_message!("welcome", {"name": "Bob", "count": "5"});
    assert_eq!(
        bundle.format_message(&msg),
        "Welcome, Bob! You have 5 messages."
    );
    cleanup_locale_dir();
}

#[tokio::test]
async fn test_fluent_format_single_value_macro() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("en_US"));
    // Single-value variant sets the key to "msg"
    let msg = fluent_message!("error-msg", "something went wrong");
    assert_eq!(bundle.format_message(&msg), "Error: something went wrong");
    cleanup_locale_dir();
}

// ─── fluent_message! macro variants ──────────────────────────────────────────

#[test]
fn test_fluent_message_macro_no_data() {
    let msg = fluent_message!("test-id");
    assert_eq!(msg.id, "test-id");
    assert_eq!(msg.crate_name, "lsys-core");
    assert!(msg.data.is_empty());
}

#[test]
fn test_fluent_message_macro_with_map() {
    let msg = fluent_message!("test-id", {"key1": "val1", "key2": "val2"});
    assert_eq!(msg.id, "test-id");
    assert_eq!(msg.crate_name, "lsys-core");
    assert_eq!(msg.data.len(), 2);
    assert_eq!(msg.data[0].0, "key1");
    assert_eq!(msg.data[1].0, "key2");
    match &msg.data[0].1 {
        FluentData::String(s) => assert_eq!(s, "val1"),
        other => panic!("Expected FluentData::String, got {:?}", other),
    }
    match &msg.data[1].1 {
        FluentData::String(s) => assert_eq!(s, "val2"),
        other => panic!("Expected FluentData::String, got {:?}", other),
    }
}

#[test]
fn test_fluent_message_macro_with_single_value() {
    let msg = fluent_message!("test-id", 42);
    assert_eq!(msg.id, "test-id");
    assert_eq!(msg.crate_name, "lsys-core");
    assert_eq!(msg.data.len(), 1);
    assert_eq!(msg.data[0].0, "msg");
    match &msg.data[0].1 {
        FluentData::String(s) => assert_eq!(s, "42"),
        other => panic!("Expected FluentData::String, got {:?}", other),
    }
}

// ─── FluentMessage::default_format ───────────────────────────────────────────

#[test]
fn test_fluent_message_default_format_empty_data() {
    let msg = fluent_message!("simple-id");
    assert_eq!(msg.default_format(), "simple-id");
}

#[test]
fn test_fluent_message_default_format_with_data() {
    let msg = fluent_message!("my-msg", {"name": "Bob", "age": "30"});
    assert_eq!(msg.default_format(), "my-msg:{name:'Bob',age:'30'}");
}

#[test]
fn test_fluent_message_default_format_nested() {
    let inner = fluent_message!("inner-id");
    let outer = lsys_core::fluents::FluentMessage {
        id: "outer-id".to_owned(),
        crate_name: "lsys-core".to_owned(),
        data: vec![("child".to_owned(), FluentData::Message(inner))],
    };
    // Nested message uses its own default_format
    assert_eq!(outer.default_format(), "outer-id:{child:'inner-id'}");
}

// ─── Chinese locale ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_fluent_format_chinese_locale() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("zh_CN"));

    let msg = fluent_message!("hello");
    assert_eq!(bundle.format_message(&msg), "你好世界");

    let msg = fluent_message!("greeting", {"name": "Alice"});
    assert_eq!(bundle.format_message(&msg), "你好, Alice!");

    cleanup_locale_dir();
}

// ─── Missing / fallback behaviour ────────────────────────────────────────────

#[tokio::test]
async fn test_fluent_missing_translation_uses_default_format() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("zh_CN"));

    // "welcome" is absent from zh_CN → falls back to default_format
    let msg = fluent_message!("welcome", {"name": "Bob", "count": "5"});
    let result = bundle.format_message(&msg);
    assert_eq!(result, msg.default_format());
    cleanup_locale_dir();
}

#[tokio::test]
async fn test_fluent_nonexistent_message_id() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    let bundle = mgr.locale(Some("en_US"));

    // Completely unknown message ID with no data → returns ID itself
    let msg = fluent_message!("nonexistent-message");
    assert_eq!(bundle.format_message(&msg), "nonexistent-message");

    cleanup_locale_dir();
}

#[tokio::test]
async fn test_fluent_locale_fallback_to_default_bundle() {
    setup_locale_dir();
    let mgr = FluentMgr::new(TEST_LOCALE_DIR, "app", Some(&["lsys-core"]))
        .await
        .unwrap();

    // Request a locale that does not exist → falls back to the empty default bundle
    let bundle = mgr.locale(Some("fr_FR"));
    let msg = fluent_message!("hello");
    // No translations available → returns the message ID
    assert_eq!(bundle.format_message(&msg), "hello");

    cleanup_locale_dir();
}
