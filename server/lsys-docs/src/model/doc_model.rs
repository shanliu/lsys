use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::SqlxModel;

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "doc_git")]
pub struct DocGitModel {
    #[sqlx(default)]
    pub id: u32,

    /// GIT地址,包含用户名
    #[sqlx(default)]
    pub url: String,

    /// 分支名
    #[sqlx(default)]
    pub branch: String,

    /// 保持最新版,0否 1 是
    #[sqlx(default)]
    pub is_update: i8,

    /// 分支是否是TAG,0否 1 是
    #[sqlx(default)]
    pub is_tag: i8,

    /// 当前使用构建版本
    #[sqlx(default)]
    pub build_version: String,

    /// 清理规则
    #[sqlx(default)]
    pub clear_rule: String,

    /// 状态:删除 正常
    #[sqlx(default)]
    pub status: i8,

    /// 第一个成功时间,修改时重置为0,更新时加build_version约束  default:  0
    #[sqlx(default)]
    pub finish_time: u64,

    /// 最后修改用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后修改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "doc_menu")]
pub struct DocMenuModel {
    #[sqlx(default)]
    pub id: u32,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_git_id: u32,

    /// 当前使用构建版本
    #[sqlx(default)]
    pub build_version: String,

    /// 目录文件路径
    #[sqlx(default)]
    pub menu_path: String,

    /// 访问路径限制
    #[sqlx(default)]
    pub access_path: String,

    /// 状态 正常 删除  
    #[sqlx(default)]
    pub status: i8,

    /// 第一个成功时间,修改时重置为0,更新时加build_version约束  default:  0
    #[sqlx(default)]
    pub finish_time: u64,

    /// 最后修改用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后修改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "doc_clone")]
pub struct DocCloneModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_git_id: u32,

    /// 克隆主机
    #[sqlx(default)]
    pub host: String,

    /// 当前使用构建版本
    #[sqlx(default)]
    pub build_version: String,

    /// 最后CLONE开始时间
    #[sqlx(default)]
    pub clone_time: u64,

    /// 克隆完成时间  default:  0
    #[sqlx(default)]
    pub finish_time: u64,

    /// 尝试克隆次数
    #[sqlx(default)]
    pub clone_try: i8,

    /// 状态:待克隆 已克隆 已删除
    #[sqlx(default)]
    pub status: i8,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "doc_build")]
pub struct DocBuildModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_git_id: u32,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_menu_id: u32,

    /// 克隆主机
    #[sqlx(default)]
    pub host: String,

    /// 当前使用构建版本
    #[sqlx(default)]
    pub build_version: String,

    /// 目录内容,仅保留成功
    #[sqlx(default)]
    pub build_data: String,

    /// 完成时间  default:  0
    #[sqlx(default)]
    pub finish_time: u64,

    /// 状态:部分完成,失败,完成 已删除
    #[sqlx(default)]
    pub status: i8,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "doc_logs")]
pub struct DocLogsModel {
    #[sqlx(default)]
    pub id: u32,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_git_id: u32,

    /// 目录配置ID,可为0  default:  0
    #[sqlx(default)]
    pub doc_menu_id: u32,

    /// 执行时主机名
    #[sqlx(default)]
    pub host: String,

    /// 当前使用构建版本
    #[sqlx(default)]
    pub build_version: String,

    /// 消息内容
    #[sqlx(default)]
    pub message: String,

    /// 最后修改时间
    #[sqlx(default)]
    pub add_time: u64,
}
