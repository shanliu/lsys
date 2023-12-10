#[tokio::test]
async fn test_ali() {
    let client = reqwest::Client::builder();
    let client = client.build().unwrap();

    let app_key = "LTAI5tD8ZcM52sTr3ubxxxxx";
    let app_secret = "alCw4PXzoPM6KFbAgaM1UGC0O2xxxx";
    let res = lsys_lib_sms::AliSms::branch_send(
        client.clone(),
        "",
        app_key,
        app_secret,
        "豆草",
        "SMS_35115133",
        r#"{"code":"1111","time":"11"}"#,
        &["13800138000"],
        "",
        "",
    )
    .await;
    println!("{:?}", res);
    let res = lsys_lib_sms::AliSms::send_detail(
        client,
        app_key,
        app_secret,
        "752509000445467877^0",
        "13800138000",
        "20231120",
    )
    .await;
    println!("{:?}", res);
    let notify_data = r#"
          [{"send_time" : "2017-08-30 00:00:00","report_time" : "2017-08-30 00:00:00","success" : true,"err_msg" : "用户接收成功","err_code" : "DELIVERED","phone_number" : "18612345678","sms_size" : "1","biz_id" : "932702304080415357^0","out_id" : "1184585343"}]
    "#;
    let res = lsys_lib_sms::AliSms::send_notify_parse(notify_data);
    println!("{:?}", res);
}
