use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use lsys_core::db::lsys_model;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "doc_git")]
pub struct DocGitModel {
    #[sqlx(default)]
    pub id: u32,

    /// GIT 名称
    #[sqlx(default)]
    pub name: String,

    /// GIT地址,包含用户名 当 doc_tag 中 tag 跟 build_version 存在源中时可替换
    #[sqlx(default)]
    pub url: String,

    /// 状态:删除 正常
    #[sqlx(default)]
    pub status: i8,

    /// 尝试次数
    #[sqlx(default)]
    pub max_try: u8,

    /// 最后修改用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后修改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "doc_tag")]
pub struct DocGitTagModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_git_id: u32,

    /// 分支名
    #[sqlx(default)]
    pub tag: String,

    /// 当前使用构建版本
    #[sqlx(default)]
    pub build_version: String,

    /// 清理规则
    #[sqlx(default)]
    pub clear_rule: String,

    /// 状态:删除 未启用 已启用[必须有一个CLONE成功]
    #[sqlx(default)]
    pub status: i8,

    /// 最后修改用户
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 最后修改时间
    #[sqlx(default)]
    pub add_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "doc_clone")]
pub struct DocGitCloneModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_tag_id: u64,

    /// 克隆主机
    #[sqlx(default)]
    pub host: String,

    /// 最后CLONE开始时间
    #[sqlx(default)]
    pub start_time: u64,

    /// 克隆完成时间  default:  0
    #[sqlx(default)]
    pub finish_time: u64,

    /// 状态:待克隆 已克隆 克隆失败 已删除[删除已克隆时,必须存在大于一个]
    /// 克隆完更新时,status!=已删除
    #[sqlx(default)]
    pub status: i8,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "doc_menu")]
pub struct DocMenuModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_tag_id: u64,

    /// 目录文件路径
    #[sqlx(default)]
    pub menu_path: String,

    /// 添加时检测主机,查问题用
    #[sqlx(default)]
    pub menu_check_host: String,

    /// 状态 正常 删除  
    #[sqlx(default)]
    pub status: i8,

    /// 最后修改用户
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 最后修改时间
    #[sqlx(default)]
    pub add_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "doc_logs")]
pub struct DocLogsModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文档GIT来源ID
    #[sqlx(default)]
    pub doc_tag_id: u64,

    /// 文件CLONE表ID
    #[sqlx(default)]
    pub doc_clone_id: u64,

    /// 执行时主机名
    #[sqlx(default)]
    pub host: String,

    /// 消息内容
    #[sqlx(default)]
    pub message: String,

    /// 最后修改时间
    #[sqlx(default)]
    pub add_time: u64,
}
