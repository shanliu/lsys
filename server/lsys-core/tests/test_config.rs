use lsys_core::config::{Config, ConfigError};
use std::fs;
use std::path::Path;

const TEST_DIR: &str = "/tmp/lsys_test_config";

fn setup_test_dir() {
    let dir = Path::new(TEST_DIR);
    if dir.exists() {
        fs::remove_dir_all(dir).unwrap();
    }
    fs::create_dir_all(dir).unwrap();

    fs::write(
        dir.join("app.toml"),
        r#"
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgres://localhost/test"
"#,
    )
    .unwrap();

    fs::write(
        dir.join("my-module.toml"),
        r#"
[server]
host = "192.168.1.1"
port = 9090

[module]
name = "my-module"
enabled = true
"#,
    )
    .unwrap();
}

fn cleanup_test_dir() {
    let _ = fs::remove_dir_all(TEST_DIR);
}

#[tokio::test]
async fn test_config_new_with_explicit_crate_list() {
    setup_test_dir();
    let config = Config::new(TEST_DIR, "app", Some(&["my-module"])).await;
    assert!(config.is_ok(), "Config::new with explicit crate list should succeed");
    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_new_with_auto_discovery() {
    setup_test_dir();
    let config = Config::new(TEST_DIR, "app", None).await;
    assert!(config.is_ok(), "Config::new with auto-discovery should succeed");

    let config = config.unwrap();
    // Auto-discovery should find my-module.toml and load its values
    let module_config = config.find(Some("my-module"));
    let host = module_config.get_string("server.host").unwrap();
    assert_eq!(host, "192.168.1.1");
    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_find_known_crate() {
    setup_test_dir();
    let config = Config::new(TEST_DIR, "app", Some(&["my-module"]))
        .await
        .unwrap();

    let module_config = config.find(Some("my-module"));

    // Module-specific values override app defaults
    let host = module_config.get_string("server.host").unwrap();
    assert_eq!(host, "192.168.1.1");

    let port = module_config.get_int("server.port").unwrap();
    assert_eq!(port, 9090);

    // Module-only value
    let name = module_config.get_string("module.name").unwrap();
    assert_eq!(name, "my-module");

    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_find_unknown_crate_falls_back_to_default() {
    setup_test_dir();
    let config = Config::new(TEST_DIR, "app", Some(&["my-module"]))
        .await
        .unwrap();

    // Unknown crate name returns the default (app) config
    let fallback = config.find(Some("nonexistent"));
    let host = fallback.get_string("server.host").unwrap();
    assert_eq!(host, "127.0.0.1");

    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_find_none_returns_default() {
    setup_test_dir();
    let config = Config::new(TEST_DIR, "app", Some(&["my-module"]))
        .await
        .unwrap();

    let default_config = config.find(None);
    let host = default_config.get_string("server.host").unwrap();
    assert_eq!(host, "127.0.0.1");

    let port = default_config.get_int("server.port").unwrap();
    assert_eq!(port, 8080);

    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_crate_config_inherits_app_values() {
    setup_test_dir();
    let config = Config::new(TEST_DIR, "app", Some(&["my-module"]))
        .await
        .unwrap();

    let module_config = config.find(Some("my-module"));

    // Value only in app.toml is inherited by the crate config
    let db_url = module_config.get_string("database.url").unwrap();
    assert_eq!(db_url, "postgres://localhost/test");

    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_error_missing_crate_file() {
    setup_test_dir();
    // Request a crate config file that does not exist on disk
    let result = Config::new(TEST_DIR, "app", Some(&["nonexistent-module"])).await;
    assert!(result.is_err(), "Missing crate config file should produce an error");

    match result {
        Err(ConfigError::Config(_)) => {}
        Err(other) => panic!("Expected ConfigError::Config, got {:?}", other),
        Ok(_) => panic!("Expected an error, but got Ok"),
    }

    cleanup_test_dir();
}

#[tokio::test]
async fn test_config_error_invalid_toml() {
    let dir = "/tmp/lsys_test_config_invalid";
    let path = Path::new(dir);
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
    fs::create_dir_all(path).unwrap();

    fs::write(path.join("app.toml"), "this is [[[not valid toml").unwrap();

    let result = Config::new(dir, "app", None).await;
    assert!(result.is_err(), "Invalid TOML should produce an error");

    match result {
        Err(ConfigError::Config(_)) => {}
        Err(other) => panic!("Expected ConfigError::Config, got {:?}", other),
        Ok(_) => panic!("Expected an error, but got Ok"),
    }

    let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn test_config_error_missing_directory() {
    let dir = "/tmp/lsys_test_config_missing_dir";
    let path = Path::new(dir);
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }

    // Explicit crate list with a non-existent base directory
    let result = Config::new(dir, "app", Some(&["module"])).await;
    assert!(
        result.is_err(),
        "Missing directory with explicit crate list should produce an error"
    );
}
