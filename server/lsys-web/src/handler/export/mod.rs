// 内置 Exporter 实现集合
//
// 每个子模块对应一类列表接口的完整导出实现：
//   - 定义 `EXPORT_TYPE_XXX` 常量，注册到 `WebExportTask` 时使用
//   - 实现 `Exporter` trait，驱动实际数据拉取与 CSV 文件生成
//   - check() 负责权限校验，export() 仅做数据拉取与 CSV 生成
mod api_system;
mod api_user;
use crate::dao::{AppSender, WebApp, WebError, WebRbac};

// ── 用户端 ──────────────────────────────────────────────────────────────────
use api_user::file::{EXPORT_TYPE_USER_FILE_LIST, FileListExporter, FileListExportCheck};
use api_user::login_history::{EXPORT_TYPE_USER_LOGIN_HISTORY, UserLoginHistoryExporter, UserLoginHistoryExportCheck};

// 用户APP
use api_user::app_collector::{
    EXPORT_TYPE_APP_SCRIPT_RECORDS, ScriptRecordsExporter,
};
use api_user::app_file::{EXPORT_TYPE_APP_FILE_LIST, AppFileListExporter, AppFileListExportCheck};
use api_user::app_notify::{EXPORT_TYPE_APP_NOTIFY_LIST, AppNotifyListExporter};
use api_user::app_res::{EXPORT_TYPE_APP_RES_DATA, AppResDataExporter, AppResDataExportCheck};
use api_user::app_role::{EXPORT_TYPE_APP_ROLE_DATA, AppRoleDataExporter, AppRoleDataExportCheck};
use api_user::app_sender_mailer::{EXPORT_TYPE_USER_MAILER_MESSAGE_LIST, MailerMessageListExporter, MailerMessageListExportCheck};
use api_user::app_sender_smser::{EXPORT_TYPE_USER_SMSER_MESSAGE_LIST, SmserMessageListExporter, SmserMessageListExportCheck};

// ── 系统管理端 ──────────────────────────────────────────────────────────────
use api_system::admin_file::{EXPORT_TYPE_SYSTEM_ADMIN_FILE_LIST, AdminFileListExporter, AdminFileListExportCheck};
use api_system::app_list::{
    EXPORT_TYPE_SYSTEM_APP_LIST, EXPORT_TYPE_SYSTEM_REQUEST_LIST, EXPORT_TYPE_SYSTEM_SUB_APP_LIST,
    SystemAppListExporter, SystemRequestListExporter, SystemSubAppListExporter,
    SystemAppListExportCheck, SystemRequestListExportCheck, SystemSubAppListExportCheck,
};
use api_system::app_sender_mailer::{
    EXPORT_TYPE_SYSTEM_MAILER_MESSAGE_LIST, SystemMailerMessageListExporter, SystemMailerMessageListExportCheck,
};
use api_system::app_sender_smser::{
    EXPORT_TYPE_SYSTEM_SMSER_MESSAGE_LIST, SystemSmserMessageListExporter, SystemSmserMessageListExportCheck,
};
use api_system::user_access::{EXPORT_TYPE_SYSTEM_USER_ACCESS, UserAccessExporter, UserAccessExportCheck};
use api_system::user_change_log::{EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG, UserChangeLogExporter, UserChangeLogExportCheck};

use lsys_access::dao::AccessDao;
use lsys_file::dao::FileDao;
use lsys_file_manager::FileCollector;
use lsys_rbac::dao::RbacDao;
use lsys_user::dao::AccountDao;
use std::sync::Arc;

/// 注册所有导出器和权限检查器到 WebExportTask
#[allow(clippy::too_many_arguments)]
pub async fn register_exporters(
    web_export_task: &mut crate::dao::WebExportTask,
    web_rbac: Arc<WebRbac>,
    account_dao: Arc<AccountDao>,
    access_dao: Arc<AccessDao>,
    web_app: Arc<WebApp>,
    app_sender: Arc<AppSender>,
    file_dao: Arc<FileDao>,
    collector: Arc<FileCollector>,
    rbac_dao: Arc<RbacDao>,
    change_logger_dao: Arc<lsys_logger::dao::ChangeLoggerDao>,
) -> Result<(), WebError> {

    // ── 用户端 ──────────────────────────────────────────────────────────────────

    // 登录历史
    web_export_task.register(
        EXPORT_TYPE_USER_LOGIN_HISTORY,
        UserLoginHistoryExporter {
            account_dao: account_dao.clone(),
            access_dao: access_dao.clone(),
        },
        UserLoginHistoryExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 文件列表
    web_export_task.register(
        EXPORT_TYPE_USER_FILE_LIST,
        FileListExporter {
            file_dao: file_dao.clone(),
        },
        FileListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // ── 用户APP ─────────────────────────────────────────────────────────────

    // 应用通知
    web_export_task.register(
        EXPORT_TYPE_APP_NOTIFY_LIST,
        AppNotifyListExporter {
            app_dao: web_app.app_dao.clone(),
        },
        api_user::app_notify::AppNotifyListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 采集器脚本执行记录
    web_export_task.register(
        EXPORT_TYPE_APP_SCRIPT_RECORDS,
        ScriptRecordsExporter {
            collector: collector.clone(),
        },
        api_user::app_collector::ScriptRecordsExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // APP 文件列表
    web_export_task.register(
        EXPORT_TYPE_APP_FILE_LIST,
        AppFileListExporter {
            file_dao: file_dao.clone(),
        },
        AppFileListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // APP RBAC 角色
    web_export_task.register(
        EXPORT_TYPE_APP_ROLE_DATA,
        AppRoleDataExporter {
            rbac_dao: rbac_dao.clone(),
        },
        AppRoleDataExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // APP RBAC 资源
    web_export_task.register(
        EXPORT_TYPE_APP_RES_DATA,
        AppResDataExporter {
            rbac_dao: rbac_dao.clone(),
        },
        AppResDataExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 邮件消息列表（用户）
    web_export_task.register(
        EXPORT_TYPE_USER_MAILER_MESSAGE_LIST,
        MailerMessageListExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
        },
        MailerMessageListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 短信消息列表（用户）
    web_export_task.register(
        EXPORT_TYPE_USER_SMSER_MESSAGE_LIST,
        SmserMessageListExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
        },
        SmserMessageListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // ── 系统管理端 ──────────────────────────────────────────────────────────

    // 系统应用列表
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_APP_LIST,
        SystemAppListExporter {
            app_dao: web_app.app_dao.clone(),
        },
        SystemAppListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    web_export_task.register(
        EXPORT_TYPE_SYSTEM_SUB_APP_LIST,
        SystemSubAppListExporter {
            app_dao: web_app.app_dao.clone(),
        },
        SystemSubAppListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    web_export_task.register(
        EXPORT_TYPE_SYSTEM_REQUEST_LIST,
        SystemRequestListExporter {
            app_dao: web_app.app_dao.clone(),
        },
        SystemRequestListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 系统邮件消息列表
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_MAILER_MESSAGE_LIST,
        SystemMailerMessageListExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
        },
        SystemMailerMessageListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 系统短信消息列表
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_SMSER_MESSAGE_LIST,
        SystemSmserMessageListExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
        },
        SystemSmserMessageListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 管理员文件列表
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_ADMIN_FILE_LIST,
        AdminFileListExporter {
            file_dao: file_dao.clone(),
        },
        AdminFileListExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 用户变更日志
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG,
        UserChangeLogExporter {
            change_logger_dao: change_logger_dao.clone(),
        },
        UserChangeLogExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    // 用户登录历史
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_USER_ACCESS,
        UserAccessExporter {
            access_dao: access_dao.clone(),
        },
        UserAccessExportCheck {
            web_rbac: web_rbac.clone(),
        },
    )?;

    Ok(())
}
