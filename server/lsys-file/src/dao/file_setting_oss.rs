use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue, TableMeta};
use lsys_core::fluent_message;
use lsys_setting::dao::{
    MultipleSetting, MultipleSettingData, SettingData, SettingDecode, SettingEncode, SettingKey,
};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

use crate::common::{FileError, FileResult};
use crate::model::{FileModel, FileStatus};
use crate::oss::OssProviderRegistry;

// ==================== 配置数据结构 ====================

/// 存储在 lsys-setting 中的 OSS 配置条目
///
/// `config_key` 作为唯一标识存入 `lst_file.storage_type`。
/// `provider_type` 标识 OSS 厂商类型（与注册表中的 key 对应）。
/// `provider_config` 存储该厂商的具体配置（原始 JSON），由注册表中的
/// 注册表负责反序列化。
/// `is_private` 标识该 OSS 存储是否为私有存储（不能生成公开访问 URL）。
///
/// 约束：`config_key` 小写字母+数字+连字符，创建后不可修改。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssSettingData {
    /// 唯一配置标识，存入 lst_file.storage_type
    pub config_key: String,
    /// OSS 厂商类型标识，如 "aliyun-oss", "aws-s3", "tencent-cos"
    pub provider_type: String,
    /// 该厂商的具体配置（原始 JSON），由对应 Factory 反序列化
    pub provider_config: serde_json::Value,
    /// 是否为私有存储（true=私有，不能生成公开访问 URL；false=公开，可以生成 URL）
    #[serde(default)]
    pub is_private: bool,
}

impl SettingKey for OssSettingData {
    fn key<'t>() -> &'t str {
        "file-oss-config"
    }
}

impl SettingDecode for OssSettingData {
    fn decode(data: &str) -> lsys_setting::dao::SettingResult<Self> {
        serde_json::from_str(data).map_err(lsys_setting::dao::SettingError::SerdeJson)
    }
}

impl SettingEncode for OssSettingData {
    fn encode(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ==================== FileOssConfigDao ====================

/// OSS 配置管理 DAO
///
/// 封装 `MultipleSetting` 的 CRUD 操作，增加：
/// - `config_key` 格式校验（小写字母+数字+连字符）
/// - `config_key` 唯一性校验（add 时分批检查现有配置）
/// - `config_key` 不可修改（edit 时检查）
/// - `provider_type` 需已在注册表中注册（add/edit 时检查）
/// - 删除前引用检查（`lst_file` 中是否有 status=1,3 的文件使用该 key）
/// - `resolve_provider`：按 config_key 查配置 → 注册表构建 OssProvider
pub struct FileOssConfigDao {
    db: Pool<MySql>,
    setting: Arc<MultipleSetting>,
    registry: Arc<OssProviderRegistry>,
    oss_config_cache: Arc<
        lsys_core::cache::LocalCache<
            String,
            Option<lsys_setting::dao::SettingData<crate::dao::OssSettingData>>,
        >,
    >,
}

/// 分批查询时每页大小
const PAGE_SIZE: u64 = 100;

impl FileOssConfigDao {
    pub fn new(
        db: Pool<MySql>,
        setting: Arc<MultipleSetting>,
        registry: Arc<OssProviderRegistry>,
        oss_config_cache: Arc<
            lsys_core::cache::LocalCache<
                String,
                Option<lsys_setting::dao::SettingData<crate::dao::OssSettingData>>,
            >,
        >,
    ) -> Self {
        Self {
            db,
            setting,
            registry,
            oss_config_cache,
        }
    }

    /// 获取注册表引用（供外部查询可用 provider 类型等）
    pub fn registry(&self) -> &OssProviderRegistry {
        &self.registry
    }

    // ==================== CRUD ====================

    /// 添加 OSS 配置
    ///
    /// - 校验 config_key 格式
    /// - 禁止使用保留值 "local"
    /// - 校验 provider_type 已注册
    /// - 分批检查 config_key 在现有配置中是否已存在
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        config_key: &str,
        provider_type: &str,
        provider_config: serde_json::Value,
        is_private: bool,
        change_user_id: u64,
        env_data: Option<&lsys_core::utils::RequestEnv>,
    ) -> FileResult<u64> {
        // 校验 config_key 格式
        Self::validate_config_key(config_key)?;

        // 禁止使用保留值（本地存储类型）
        if FileModel::is_local_key(config_key) {
            return Err(FileError::Param(fluent_message!(
                "oss-config-key-reserved",
                {"key": config_key}
            )));
        }

        // 校验 provider_type 已注册
        if !self.registry.has_type(provider_type) {
            return Err(FileError::Param(fluent_message!(
                "oss-provider-type-unknown",
                {"type": provider_type}
            )));
        }

        // 检查 config_key 唯一性：分批遍历现有配置
        self.check_config_key_unique(config_key, None).await?;

        let data = OssSettingData {
            config_key: config_key.to_string(),
            provider_type: provider_type.to_string(),
            provider_config,
            is_private,
        };

        let id = self
            .setting
            .add::<OssSettingData>(
                None,
                &MultipleSettingData { name, data: &data },
                change_user_id,
                None,
                env_data,
            )
            .await?;

        // 清理缓存
        self.oss_config_cache.clear(&config_key.to_string()).await;

        Ok(id)
    }

    /// 修改 OSS 配置
    ///
    /// 只允许修改 `name`、`provider_config`（认证信息等）和 `is_private`。
    /// `config_key` 和 `provider_type` 创建后不可变，无需传入。
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        provider_config: serde_json::Value,
        is_private: bool,
        change_user_id: u64,
        env_data: Option<&lsys_core::utils::RequestEnv>,
    ) -> FileResult<u64> {
        // 加载旧配置，保留 config_key 和 provider_type
        let old = self.setting.load::<OssSettingData>(None, id).await?;

        let data = OssSettingData {
            config_key: old.config_key.clone(),
            provider_type: old.provider_type.clone(),
            provider_config,
            is_private,
        };

        let rows = self
            .setting
            .edit::<OssSettingData>(
                None,
                id,
                &MultipleSettingData { name, data: &data },
                change_user_id,
                None,
                env_data,
            )
            .await?;

        // 清理缓存
        self.oss_config_cache.clear(&old.config_key).await;

        Ok(rows)
    }

    /// 删除 OSS 配置
    ///
    /// 删除前检查：如有活跃文件（status=Normal 或 Unfinished）使用该 config_key，拒绝删除
    pub async fn del_config(
        &self,
        id: u64,
        change_user_id: u64,
        env_data: Option<&lsys_core::utils::RequestEnv>,
    ) -> FileResult<u64> {
        // 先加载配置获取 config_key
        let config = self.setting.load::<OssSettingData>(None, id).await?;

        // 引用检查
        let count = self.active_file_count(&config.config_key).await?;

        if count > 0 {
            return Err(FileError::Param(fluent_message!(
                "oss-config-in-use",
                {"key": &config.config_key, "count": count}
            )));
        }

        let rows = self
            .setting
            .del::<OssSettingData>(None, id, change_user_id, None, env_data)
            .await?;

        // 清理缓存
        self.oss_config_cache.clear(&config.config_key).await;

        Ok(rows)
    }

    /// 列表查询
    pub async fn list_config(
        &self,
        page: &OffsetPageParam,
    ) -> FileResult<Vec<SettingData<OssSettingData>>> {
        Ok(self
            .setting
            .list_data::<OssSettingData>(None, None, page)
            .await?)
    }

    /// 配置总数
    pub async fn list_count(&self) -> FileResult<i64> {
        Ok(self.setting.list_count::<OssSettingData>(None).await?)
    }

    /// 按 ID 加载单条配置
    pub async fn load_config(&self, id: u64) -> FileResult<SettingData<OssSettingData>> {
        Ok(self.setting.load::<OssSettingData>(None, id).await?)
    }

    /// 按 config_key 查找配置（分批遍历）
    pub async fn find_by_config_key(
        &self,
        config_key: &str,
    ) -> FileResult<Option<SettingData<OssSettingData>>> {
        let mut offset: u64 = 0;
        loop {
            let page = OffsetPageParam::new(Some(OffsetPageValue::new(offset, PAGE_SIZE)));
            let batch = self
                .setting
                .list_data::<OssSettingData>(None, None, &page)
                .await?;

            if batch.is_empty() {
                return Ok(None);
            }

            for item in &batch {
                if item.config_key == config_key {
                    return Ok(Some(item.clone()));
                }
            }

            if (batch.len() as u64) < PAGE_SIZE {
                return Ok(None);
            }
            offset += PAGE_SIZE;
        }
    }

    /// 根据 config_key 获取 OssProvider（无缓存，每次从 DB 读取）
    ///
    /// 从 lsys-setting 按 key 查配置 → 注册表 build_provider
    pub async fn resolve_provider(
        &self,
        config_key: &str,
    ) -> FileResult<Box<dyn crate::common::OssProvider>> {
        let config_data = self.find_by_config_key(config_key).await?.ok_or_else(|| {
            FileError::Param(fluent_message!(
                "oss-config-not-found",
                {"key": config_key}
            ))
        })?;

        self.registry
            .build_provider(
                &config_data.provider_type,
                config_data.provider_config.clone(),
            )
            .await
    }

    // ==================== 内部方法 ====================

    /// 检查 lst_file 中使用该 config_key 的活跃文件数量
    async fn active_file_count(&self, config_key: &str) -> FileResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(&format!(
            "SELECT COUNT(*) FROM {} WHERE storage_type=? AND status IN (?,?)",
            FileModel::table_name(),
        ))
        .bind(config_key)
        .bind(FileStatus::Normal as i8)
        .bind(FileStatus::Unfinished as i8)
        .fetch_one(&self.db)
        .await?;
        Ok(count)
    }

    /// 检查 config_key 在现有配置中是否唯一（分批遍历）
    ///
    /// `exclude_id`: 编辑时排除自身 ID
    async fn check_config_key_unique(
        &self,
        config_key: &str,
        exclude_id: Option<u64>,
    ) -> FileResult<()> {
        let mut offset: u64 = 0;
        loop {
            let page = OffsetPageParam::new(Some(OffsetPageValue::new(offset, PAGE_SIZE)));
            let batch = self
                .setting
                .list_data::<OssSettingData>(None, None, &page)
                .await?;

            if batch.is_empty() {
                return Ok(());
            }

            for item in &batch {
                if item.config_key == config_key {
                    if let Some(eid) = exclude_id
                        && item.model().id == eid
                    {
                        continue;
                    }
                    return Err(FileError::Param(fluent_message!(
                        "oss-config-key-exists",
                        {"key": config_key}
                    )));
                }
            }

            if (batch.len() as u64) < PAGE_SIZE {
                return Ok(());
            }
            offset += PAGE_SIZE;
        }
    }

    /// 校验 config_key 格式
    ///
    /// 规则：
    /// - 长度 1~32
    /// - 只允许小写字母、数字、连字符
    /// - 不能以连字符开头或结尾
    fn validate_config_key(key: &str) -> FileResult<()> {
        use lsys_core::valid_key;
        use lsys_core::valid_param::{
            ValidParam, ValidParamCheck, ValidPattern, ValidStrMatch, ValidStrlen,
        };
        ValidParam::default()
            .add(
                valid_key!("config_key"),
                &key,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, 32))
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrMatch::StartNotWith("-"))
                    .add_rule(ValidStrMatch::EndNotWith("-")),
            )
            .check()?;
        Ok(())
    }
}
