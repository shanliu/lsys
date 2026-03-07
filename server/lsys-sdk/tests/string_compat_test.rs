//! 测试服务器返回字符串类型时的自动兼容处理
//!
//! 本测试验证 lsys-sdk 可以正确处理服务器将 BOOL、INT 等类型转为字符串输出的情况

use lsys_sdk::{
    AppFeatureResponse, AppSecretResponse, AuthVerifyResponse, FileInfoItem, FileListItem,
    FileListResponse, FileUploadByMd5Response, FileUploadCreateResponse, RbacCheckResponse,
    RbacCheckStatus,
};
use serde_json::json;

#[test]
fn test_bool_from_string_true() {
    let json_data = json!({
        "enabled": "1",
        "app_user_id": "123",
        "denied_keys": []
    });

    let response: AppFeatureResponse = serde_json::from_value(json_data).unwrap();
    assert!(response.enabled);
    assert_eq!(response.app_user_id, 123);
}

#[test]
fn test_bool_from_string_false() {
    let json_data = json!({
        "enabled": "0",
        "app_user_id": "456",
        "denied_keys": []
    });

    let response: AppFeatureResponse = serde_json::from_value(json_data).unwrap();
    assert!(!response.enabled);
    assert_eq!(response.app_user_id, 456);
}

#[test]
fn test_bool_from_actual_bool() {
    let json_data = json!({
        "enabled": true,
        "app_user_id": 789,
        "denied_keys": []
    });

    let response: AppFeatureResponse = serde_json::from_value(json_data).unwrap();
    assert!(response.enabled);
    assert_eq!(response.app_user_id, 789);
}

#[test]
fn test_u64_from_string() {
    let json_data = json!({
        "user_id": "1234567890",
        "app_id": "9876543210",
        "nickname": null,
        "username": null,
        "userdata": null
    });

    let response: AuthVerifyResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.user_id, 1234567890);
    assert_eq!(response.app_id, 9876543210);
}

#[test]
fn test_u64_from_number() {
    let json_data = json!({
        "user_id": 1234567890u64,
        "app_id": 9876543210u64,
        "nickname": null,
        "username": null,
        "userdata": null
    });

    let response: AuthVerifyResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.user_id, 1234567890);
    assert_eq!(response.app_id, 9876543210);
}

#[test]
fn test_mixed_types_file_upload_create() {
    let json_data = json!({
        "file_user_id": "654321",
        "file_id": "123456",
        "file_name": "test.txt",
        "status": "1",
        "upload_token": "token123"
    });

    let response: FileUploadCreateResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.file_user_id, 654321);
    assert_eq!(response.file_id, 123456);
    assert_eq!(response.status, 1);
    assert_eq!(response.upload_token, "token123");
}

#[test]
fn test_mixed_types_from_numbers() {
    let json_data = json!({
        "file_user_id": 654321,
        "file_id": 123456,
        "file_name": "test.txt",
        "status": 1,
        "upload_token": "token123"
    });

    let response: FileUploadCreateResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.file_user_id, 654321);
    assert_eq!(response.file_id, 123456);
    assert_eq!(response.status, 1);
}

#[test]
fn test_bool_matched_from_string() {
    let json_data = json!({
        "matched": "1",
        "file_user_id": "789"
    });

    let response: FileUploadByMd5Response = serde_json::from_value(json_data).unwrap();
    assert!(response.matched);
    assert_eq!(response.file_user_id, Some(789));
}

#[test]
fn test_rbac_check_response_bool_from_string() {
    let json_data = json!({
        "result": [
            {
                "status": "1",
                "name": "admin_panel"
            },
            {
                "status": "0",
                "name": "user_management"
            }
        ]
    });

    let response: RbacCheckResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.result.len(), 2);
    assert!(response.result[0].status);
    assert_eq!(response.result[0].name, "admin_panel");
    assert!(!response.result[1].status);
    assert_eq!(response.result[1].name, "user_management");
}

#[test]
fn test_rbac_check_status_from_bool() {
    let json_data = json!({
        "status": true,
        "name": "test_item"
    });

    let item: RbacCheckStatus = serde_json::from_value(json_data).unwrap();
    assert!(item.status);
    assert_eq!(item.name, "test_item");
}

#[test]
fn test_file_list_item_all_strings() {
    let json_data = json!({
        "id": "100",
        "file_user_id": "200",
        "file_name": "document.pdf",
        "file_md5": "abc123",
        "file_size": "1024000",
        "storage_type": "local",
        "status": "2",
        "content_type": "application/pdf",
        "source_url": "",
        "file_url": null,
        "add_time": "1678901234",
        "user_id": "300",
        "local_path": null,
        "tags": null
    });

    let item: FileListItem = serde_json::from_value(json_data).unwrap();
    assert_eq!(item.id, 100);
    assert_eq!(item.file_user_id, 200);
    assert_eq!(item.file_size, 1024000);
    assert_eq!(item.status, 2);
    assert_eq!(item.add_time, 1678901234);
    assert_eq!(item.user_id, 300);
}

#[test]
fn test_file_list_response_with_optional_fields() {
    let json_data = json!({
        "data": [],
        "next_cursor": "500",
        "prev_cursor": "100",
        "total": "42"
    });

    let response: FileListResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.next_cursor, Some(500));
    assert_eq!(response.prev_cursor, Some(100));
    assert_eq!(response.total, Some(42));
}

#[test]
fn test_file_list_response_without_optional_fields() {
    let json_data = json!({
        "data": []
    });

    let response: FileListResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.next_cursor, None);
    assert_eq!(response.prev_cursor, None);
    assert_eq!(response.total, None);
}

#[test]
fn test_app_secret_response_time_out_from_string() {
    let json_data = json!({
        "app_id": "12345",
        "user_id": "67890",
        "secrets": [
            {
                "secret_data": {"key": "value"},
                "time_out": "3600"
            }
        ]
    });

    let response: AppSecretResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.app_id, 12345);
    assert_eq!(response.user_id, 67890);
    assert_eq!(response.secrets.len(), 1);
    assert_eq!(response.secrets[0].time_out, 3600);
}

#[test]
fn test_file_info_item_i8_status_from_string() {
    let json_data = json!({
        "file_user_id": "111",
        "file_id": "222",
        "file_name": "image.png",
        "file_md5": "def456",
        "file_size": "2048000",
        "status": "-1",
        "file_url": "http://example.com/image.png",
        "storage_type": "oss",
        "content_type": "image/png"
    });

    let item: FileInfoItem = serde_json::from_value(json_data).unwrap();
    assert_eq!(item.file_user_id, 111);
    assert_eq!(item.file_id, 222);
    assert_eq!(item.file_size, 2048000);
    assert_eq!(item.status, -1);
}

#[test]
fn test_negative_i8_from_number() {
    let json_data = json!({
        "file_user_id": 111,
        "file_id": 222,
        "file_name": "test.txt",
        "file_md5": "abc",
        "file_size": 100,
        "status": -1,
        "file_url": "http://example.com/test.txt",
        "storage_type": "local",
        "content_type": "text/plain"
    });

    let item: FileInfoItem = serde_json::from_value(json_data).unwrap();
    assert_eq!(item.status, -1);
}

#[test]
fn test_option_i64_total_from_string() {
    let json_data = json!({
        "data": [],
        "next_cursor": null,
        "prev_cursor": null,
        "total": "-1"
    });

    let response: FileListResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.total, Some(-1));
}

#[test]
fn test_option_empty_string_as_none() {
    // 空字符串应该被视为 None
    let json_data = json!({
        "data": [],
        "next_cursor": "",
        "prev_cursor": "",
        "total": ""
    });

    let response: FileListResponse = serde_json::from_value(json_data).unwrap();
    assert_eq!(response.next_cursor, None);
    assert_eq!(response.prev_cursor, None);
    assert_eq!(response.total, None);
}
