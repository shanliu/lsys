use lsys_actix_web::create_server;
use std::path::PathBuf;
use std::str::FromStr;

#[actix_web::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    let app_dir = if PathBuf::from_str("./").unwrap().join("./config").exists() {
        "./"
    } else {
        //on dev
        env!("CARGO_MANIFEST_DIR")
    };

    let app = create_server(app_dir).await.unwrap();
    app.await.unwrap();
}
