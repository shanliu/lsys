use crate::dao::WebDao;
use std::path::PathBuf;

/// 返回本地公开文件存储目录路径（local_public storage）
///
/// 该路径可用于在 HTTP 服务器中挂载静态文件服务，将存储目录暴露为公开访问路径。
/// 相对路径会基于 app_path 解析为绝对路径。
pub fn local_public_dir(web_dao: &WebDao) -> PathBuf {
    let raw = web_dao.web_file.file_dao.config().public_dir().to_string();
    let path = PathBuf::from(&raw);
    if path.is_absolute() {
        path
    } else {
        web_dao.app_core.app_path.join(path)
    }
}
