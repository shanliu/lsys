use lsys_actix_web::create_server;
use std::path::PathBuf;
use std::str::FromStr;

#[actix_web::main]
async fn main() {
    let app_dir = if PathBuf::from_str("./").unwrap().join("./config").exists() {
        "./"
    } else {
        //on dev
        env!("CARGO_MANIFEST_DIR")
    };
    create_server(app_dir, &["config/app.toml"])
        .await
        .unwrap()
        .await
        .unwrap();
}
