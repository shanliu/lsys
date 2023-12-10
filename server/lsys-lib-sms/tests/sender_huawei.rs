#[tokio::test]
async fn testhw() {
    let client = reqwest::Client::builder();
    let client = client.build().unwrap();

    let res = lsys_lib_sms::HwSms::branch_send(
        client,
        "https://smsapi.ap-southeast-1.myhuaweicloud.com",
        "20b1a82bae084ce08102c396290000",
        "20b1a82bae084ce08102c396290000",
        "签名",
        "from",
        "template_id",
        lsys_lib_sms::template_map_to_arr(r#"{"aa":"01","bb":"02"}"#, "aa,bb,aa"),
        &["13800138000", "13800138001"],
        "http://127.0.0.1:8081",
        "",
    )
    .await;
    println!("{:?}", res);
    let mut map = std::collections::HashMap::new();
    map.insert("status".to_owned(), "DELIVRD".to_owned());
    map.insert("smsMsgId".to_owned(), "xxxxxxxxxxx".to_owned());
    map.insert("updateTime".to_owned(), "2018-04-13T06:31:46Z".to_owned());
    let res = lsys_lib_sms::HwSms::send_notify_parse(&map);
    println!("{:?}", res);
}
