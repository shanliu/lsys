#[tokio::test]
async fn testjd() {
    let client = reqwest::Client::builder();
    let client = client.build().unwrap();

    let res = lsys_lib_sms::JdSms::branch_send(
        client.clone(),
        false,
        "cn-north-1",
        "JDC_6FAA1A3057BA9E27760000000",
        "25D9BDBC3D21C4AF7D00000000000",
        "签名",
        "template_id",
        lsys_lib_sms::template_map_to_arr(r#"{"aa":"01","bb":"02"}"#, "aa,bb,aa"),
        &["13800138000", "13800138001"],
    )
    .await;
    println!("{:?}", res);

    let res = lsys_lib_sms::JdSms::send_detail(
        client,
        false,
        "cn-north-1",
        "JDC_6FAA1A3057BA9E27760000000",
        "25D9BDBC3D21C4AF7D00000000000",
        "send_id",
        None,
    )
    .await;
    println!("{:?}", res);
}
