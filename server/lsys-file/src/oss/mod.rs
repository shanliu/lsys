#[cfg(feature = "aliyun-oss")]
pub mod aliyun;

#[cfg(feature = "aws-s3")]
pub mod aws;

#[cfg(feature = "tencent-cos")]
pub mod tencent;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::common::{FileError, FileResult, OssProvider};

/// 注册表内部的构建函数类型
type ProviderBuilderFn = Box<
    dyn Fn(
            serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = FileResult<Box<dyn OssProvider>>> + Send>>
        + Send
        + Sync,
>;

/// 动态 OSS Provider 注册表
///
/// 按 `provider_type` 字符串索引，存储构建闭包。
/// 外部通过 `register::<T>()` 添加自定义 provider，
/// 或通过 `register_builder()` 直接注册构建函数。
///
/// 新增厂商步骤：
/// 1. 实现 `OssProviderConfig + OssProvider`
/// 2. 调用 `registry.register::<MyConfig>()` 即可 — 库内、库外均可
pub struct OssProviderRegistry {
    builders: HashMap<String, ProviderBuilderFn>,
}

impl OssProviderRegistry {
    /// 空注册表
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
        }
    }
}

impl Default for OssProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OssProviderRegistry {
    /// 注册一个 Config → Provider 配对
    ///
    /// `P::provider_type()` 自动作为注册表的 key。
    ///
    /// ```ignore
    /// registry.register::<AliyunOssConfig, AliyunOssProvider>();
    /// ```
    pub fn register<C, P>(&mut self)
    where
        C: crate::common::OssProviderConfig + serde::de::DeserializeOwned + Send + 'static,
        P: OssProvider,
    {
        self.add_builder(
            P::provider_type(),
            Box::new(|json| {
                Box::pin(async move {
                    let config: C = serde_json::from_value(json).map_err(|e| {
                        FileError::System(lsys_core::fluent_message!("oss-config-parse-error", e))
                    })?;
                    config.build_provider().await
                })
            }),
        );
    }

    /// 内部统一插入方法
    fn add_builder(&mut self, provider_type: &str, builder: ProviderBuilderFn) {
        self.builders.insert(provider_type.to_string(), builder);
    }

    /// 根据 provider_type + JSON 配置构造 OssProvider
    pub async fn build_provider(
        &self,
        provider_type: &str,
        config_json: serde_json::Value,
    ) -> FileResult<Box<dyn OssProvider>> {
        let builder = self.builders.get(provider_type).ok_or_else(|| {
            FileError::Param(lsys_core::fluent_message!(
                "oss-provider-type-unknown",
                {"type": provider_type}
            ))
        })?;
        builder(config_json).await
    }

    /// 返回所有已注册的 provider_type 列表（供前端下拉选择）
    pub fn available_types(&self) -> Vec<&str> {
        self.builders.keys().map(|s| s.as_str()).collect()
    }

    /// 检查某 provider_type 是否已注册
    pub fn has_type(&self, provider_type: &str) -> bool {
        self.builders.contains_key(provider_type)
    }

    /// 创建包含所有编译期启用的内置 provider 的注册表
    pub fn with_defaults() -> Self {
        #[allow(unused_mut)]
        let mut registry = Self::new();

        #[cfg(feature = "aliyun-oss")]
        registry.register::<aliyun::AliyunOssConfig, aliyun::AliyunOssProvider>();

        #[cfg(feature = "aws-s3")]
        registry.register::<aws::AwsOssConfig, aws::AwsOssProvider>();

        #[cfg(feature = "tencent-cos")]
        registry.register::<tencent::TencentCosConfig, tencent::TencentCosProvider>();

        registry
    }
}
