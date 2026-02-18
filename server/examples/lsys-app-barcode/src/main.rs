mod dao;
mod handler;
mod model;
mod server;
mod utils;

pub use utils::auth;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(err) = server::run().await {
        tracing::error!("server start failed: {err}");
    }
}
