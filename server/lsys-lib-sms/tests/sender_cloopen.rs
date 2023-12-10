#[tokio::test]
async fn test_cc() {
    let client = reqwest::Client::builder();
    let client = client.build().unwrap();

    let res = lsys_lib_sms::CloOpenSms::branch_send(
        client.clone(),
        "2c94811c8b1e335b018bea7700000",
        "20b1a82bae084ce08102c396290000",
        "2c94811c8b1e335b018bea77000",
        "1", //template_id
        lsys_lib_sms::template_map_to_arr(r#"{"aa":"100","bb":"20"}"#, "aa,bb"),
        &["13800138000"],
    )
    .await;
    println!("{:?}", res);
    let res = lsys_lib_sms::CloOpenSms::send_detail(
        client,
        "2c94811c8b1e335b018bea77ea000",
        "20b1a82bae084ce08102c39629000",
        "2c94811c8b1e335b018bea77e000",
    )
    .await;
    println!("{:?}", res);
    let res = lsys_lib_sms::CloOpenSms::send_notify_parse(
        r#"{
                "Request": {
                "action": "SMSArrived",
                "smsType": "1",
                "apiVersion": "2013-12-26",
                "content": "4121908f3d1b4edb9210f0eb4742f62c",
                "fromNum": "13912345678",
                "dateSent": "20130923010000",
                "deliverCode": "DELIVRD",
                "recvTime": "20130923010010",
                "status": "0",
                "reqId": "123",
                "smsCount": "2",
                "spCode": "10690876"
                }
            }"#,
    );
    println!("{:?}", res);
}
