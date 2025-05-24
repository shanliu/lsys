use lsys_web::lsys_core::AppCoreError;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::CertificateDer;
use rustls::pki_types::PrivateKeyDer;
use rustls::server::ServerConfig;
use tracing::debug;

use super::result::AppError;

pub(crate) fn load_rustls_config(
    app_dir: &str,
    cert_path: &str,
    key_path: &str,
) -> Result<ServerConfig, AppError> {
    // init server config builder with safe defaults
    let config = ServerConfig::builder().with_no_client_auth();

    let cert_path = if cert_path.strip_suffix('/').is_some() {
        cert_path.to_string()
    } else {
        app_dir.to_string().trim_end_matches('/').to_owned() + "/" + cert_path
    };
    debug!("ssl cert path :{}", &cert_path);
    let cert_chain_tmp = CertificateDer::pem_file_iter(&cert_path)
        .map_err(|e| AppCoreError::System(format!("read  cert fail: {} [{}]", e, cert_path)))?;
    let mut cert_chain = vec![];
    for cert in cert_chain_tmp {
        cert_chain.push(cert.map_err(|e| {
            AppCoreError::System(format!("parse cert fail: {} [{}]", e, cert_path))
        })?);
    }
    let key_path = if key_path.strip_suffix('/').is_some() {
        key_path.to_string()
    } else {
        app_dir.to_string().trim_end_matches('/').to_owned() + "/" + key_path
    };
    debug!("ssl key path :{}", &key_path);
    let sec1 = PrivateKeyDer::from_pem_file(&key_path).map_err(|e| {
        AppCoreError::System(format!("failed to parse cert: {} [{}]", e, cert_path))
    })?;
    Ok(config.with_single_cert(cert_chain, sec1)?)
}
