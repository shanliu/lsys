#[tokio::test]
async fn test_emay() {
    let client = reqwest::Client::builder().build().unwrap();

    // 替换为真实的服务地址、AppID 和 secretKey
    let host = "http://ip:port";
    let app_id = "your_app_id";
    let secret_key = "your_secret_key_16"; // 16 字节 AES-128 密钥

    // 批量发送（非自定义 SMSID）
    let res = lsys_lib_sms::EmaySms::branch_send(
        client.clone(),
        host,
        app_id,
        secret_key,
        "【测试公司】您的验证码是123456",
        &["13800138000", "13800138001"],
        "", // 扩展码（选填）
    )
    .await;
    println!("branch_send: {:?}", res);

    // 批量发送（自定义 SMSID）
    let res = lsys_lib_sms::EmaySms::branch_send_custom(
        client.clone(),
        host,
        app_id,
        secret_key,
        "【测试公司】您的验证码是654321",
        &["13800138000", "13800138001"],
        Some(vec![
            "custom_id_00001".to_string(),
            "custom_id_00002".to_string(),
        ]),
        "",
    )
    .await;
    println!("branch_send_custom: {:?}", res);

    // 解析状态报告普通推送回调
    // 此数据对应 HTTP POST 参数 `reports` 的值
    let notify_data = r#"[{
        "mobile":"15538850000",
        "smsId":"20170392833833891100",
        "customSmsId":"1553885000011111",
        "state":"DELIVRD",
        "desc":"成功",
        "receiveTime":"2017-03-15 12:00:00",
        "submitTime":"2017-03-15 12:00:00",
        "extendedCode":"123"
    },{
        "mobile":"15538850001",
        "smsId":"20170392833833891101",
        "customSmsId":"1553885000011112",
        "state":"UNDELIV",
        "desc":"用户不在服务区",
        "receiveTime":"2017-03-15 12:01:00",
        "submitTime":"2017-03-15 12:00:30",
        "extendedCode":"123"
    }]"#;
    let res = lsys_lib_sms::EmaySms::send_notify_parse(notify_data);
    println!("send_notify_parse: {:?}", res);

    // 回调响应输出（成功场景，响应体应为 "success"）
    let output = lsys_lib_sms::EmaySms::send_notify_output(&Ok(()));
    println!("send_notify_output (ok): {}", output);
    assert_eq!(output, "success");

    // 回调响应输出（失败场景）
    let output = lsys_lib_sms::EmaySms::send_notify_output(&Err("处理失败".to_string()));
    println!("send_notify_output (err): {}", output);
}
