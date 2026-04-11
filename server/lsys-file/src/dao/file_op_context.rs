use std::sync::OnceLock;

use crate::common::{FileError, FileResult, OssProvider};
use crate::model::{FileModel, FileUserModel};
use lsys_core::fluent_message;

use super::file_helpers::FileHelper;
use super::file_oss_config::FileOssConfigDao;

/// 文件操作上下文
///
/// 统一封装 `file_user` / `file` / `oss_provider` 三级参数，
/// 调用方按需提供，避免各方法签名中重复出现 3~4 个同质参数。
///
/// `helper` 和 `oss_config` 在构造时传入，之后 `file()` / `oss_provider()`
/// 获取时自动按需加载，无需外部手动调用 load。
///
/// 内部使用 `OnceLock` 实现延迟加载 + 缓存，`file()` / `oss_provider()`
/// 均为 `&self`（共享借用），可同时持有两者的引用而不冲突。
///
/// # 构造方式
///
/// ```ignore
/// // 通过 FileDao 创建（推荐）
/// let ctx = file_dao.create_context(&file_user).with_file(&file)?;
///
/// // file() / oss_provider() 首次调用自动加载，后续命中缓存
/// let file = ctx.file().await?;
/// let oss  = ctx.oss_provider().await?;
/// // 两者可同时使用，无借用冲突
/// ```
pub struct FileOpContext<'a> {
    pub file_user: &'a FileUserModel,
    helper: &'a FileHelper,
    oss_config: &'a FileOssConfigDao,
    file_external: Option<&'a FileModel>,
    file_loaded: OnceLock<FileModel>,
    oss_provider_external: Option<&'a dyn OssProvider>,
    oss_provider_loaded: OnceLock<Box<dyn OssProvider>>,
}

impl<'a> FileOpContext<'a> {
    pub fn new(
        file_user: &'a FileUserModel,
        helper: &'a FileHelper,
        oss_config: &'a FileOssConfigDao,
    ) -> Self {
        Self {
            file_user,
            helper,
            oss_config,
            file_external: None,
            file_loaded: OnceLock::new(),
            oss_provider_external: None,
            oss_provider_loaded: OnceLock::new(),
        }
    }

    /// 附加 FileModel（避免内部重复查询）
    ///
    /// 校验 `file.id == file_user.file_id`，不一致时返回错误。
    pub fn with_file(mut self, file: &'a FileModel) -> FileResult<Self> {
        if file.id != self.file_user.file_id {
            return Err(FileError::Param(fluent_message!(
                "file-id-mismatch",
                {"file_id": file.id, "file_user_file_id": self.file_user.file_id}
            )));
        }
        self.file_external = Some(file);
        Ok(self)
    }

    /// 附加 OssProvider（手动指定）
    ///
    /// 校验：
    /// 1. 若已知 file，file 不能是 local 类型
    /// 2. 若已知 file，provider 的类型必须与 file 关联的配置中的 `provider_type` 一致
    pub async fn with_oss_provider<P: OssProvider>(mut self, provider: &'a P) -> FileResult<Self> {
        if let Some(f) = self.file_external.or(self.file_loaded.get()) {
            if f.is_local() {
                return Err(FileError::Param(fluent_message!("file-must-be-oss-type")));
            }
            if let Some(config_key) = f.oss_config_key() {
                let config_data = self
                    .oss_config
                    .find_by_config_key(config_key)
                    .await?
                    .ok_or_else(|| {
                        FileError::Param(fluent_message!(
                            "oss-config-not-found",
                            {"key": config_key}
                        ))
                    })?;
                if config_data.provider_type != P::provider_type() {
                    return Err(FileError::Param(fluent_message!(
                        "oss-provider-type-mismatch",
                        {
                            "expected": config_data.provider_type.as_str(),
                            "actual": P::provider_type()
                        }
                    )));
                }
            }
        }
        self.oss_provider_external = Some(provider);
        Ok(self)
    }

    /// 获取 FileModel，未缓存时自动从 DB 加载
    ///
    /// 基于 `OnceLock`，首次调用查询数据库，后续命中缓存。
    /// 取 `&self`（共享借用），可与 `oss_provider()` 同时持有返回引用。
    pub async fn file(&self) -> FileResult<&FileModel> {
        if let Some(f) = self.file_external {
            return Ok(f);
        }
        if let Some(f) = self.file_loaded.get() {
            return Ok(f);
        }
        let file = self
            .helper
            .find_file_by_id(self.file_user.file_id)
            .await?
            .ok_or_else(|| FileError::Param(fluent_message!("file-not-found")))?;
        Ok(self.file_loaded.get_or_init(|| file))
    }

    /// 获取 OssProvider，未缓存时自动从注册表解析
    ///
    /// 内部会先确保 file 已加载，然后根据 `file.oss_config_key()` 解析 provider。
    /// file 为 local 类型时报错。
    pub async fn oss_provider(&self) -> FileResult<&dyn OssProvider> {
        if let Some(p) = self.oss_provider_external {
            return Ok(p);
        }
        if let Some(p) = self.oss_provider_loaded.get() {
            return Ok(p.as_ref());
        }
        let file = self.file().await?;
        let config_key = file
            .oss_config_key()
            .ok_or_else(|| FileError::Param(fluent_message!("file-must-be-oss-type")))?;
        let provider = self.oss_config.resolve_provider(config_key).await?;
        Ok(self.oss_provider_loaded.get_or_init(|| provider).as_ref())
    }
}
