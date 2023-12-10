#[tokio::test]
async fn test_ns() {
    let client = reqwest::Client::builder();
    let client = client.build().unwrap();
    let app_key = "d8a032e031b2f7cf9700000000";
    let app_secret = "f8b2f7500000";

    let res = lsys_lib_sms::NeteaseSms::branch_send(
        client.clone(),
        app_key,
        app_secret,
        "22521474",
        lsys_lib_sms::template_map_to_arr(r#"{"aa":"01"}"#, "aa"),
        &["13800138000", "13800138001"],
    )
    .await;
    println!("{:?}", res);
    let res = lsys_lib_sms::NeteaseSms::send_detail(client, app_key, app_secret, "1491").await;
    println!("{:?}", res);

    let notify_data = r#"{ "eventType": "11", "objects": [ {  "mobile": "12345678945",  "sendid": "1490",  "result": "DELIVRD",  "sendTime": "2017-06-02 14:40:45",  "reportTime": "2017-06-06 10:40:30",  "spliced": "1","templateId":1234 }, {  "mobile": "12345678945",  "sendid": "1491",  "result": "DELIVRD",  "sendTime": "2017-06-02 14:41:00",  "reportTime": "2017-06-02 10:41:20",  "spliced": "2" ,"templateId":1234} ]}"#;
    let header_md5 = "2d35ef62d088aa6a176ab5e92e30a967";
    let header_curtime = "2017-06-02 14:40:45";
    let header_checksum = "8aee9fa350c6dc7081129794882b4d16bf1034e2";
    let res = lsys_lib_sms::NeteaseSms::send_notify_parse(
        notify_data,
        //  None,
        Some((app_secret, header_md5, header_curtime, header_checksum)),
    );
    println!("{:?}", res);
}
