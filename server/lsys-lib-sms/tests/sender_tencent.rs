#[tokio::test]
async fn test_tensms() {
    let client = reqwest::Client::builder();
    let client = client.build().unwrap();
    let res = lsys_lib_sms::TenSms::branch_send(
        client.clone(),
        "ap-beijing",
        "AKIDNdmiTzNakkaGlFbqz200000000000",
        "FPUUSMB1JcvBTn8SgcWM5000000",
        "1400006666",
        "腾讯云",
        "1110",
        lsys_lib_sms::template_map_to_arr(r#"{"aa":"01","bb":"02"}"#, "aa,bb,aa"),
        &["13800138000", "13800138001"],
    )
    .await;
    println!("{:?}", res);

    let res = lsys_lib_sms::TenSms::send_detail(
        client,
        "ap-beijing",
        "AKIDNdmiTzNakkaGlFbqz200000000000",
        "FPUUSMB1JcvBTn8SgcWM5000000",
        "1400006666",
        "13800138000",
        "2023-11-15",
    )
    .await;
    println!("{:?}", res);
}
