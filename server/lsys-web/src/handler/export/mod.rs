// 内置 Exporter 实现集合
//
// 每个子模块对应一类列表接口的完整导出实现：
//   - 定义 `EXPORT_TYPE_XXX` 常量，注册到 `WebExportTask` 时使用
//   - 实现 `Exporter` trait，驱动实际数据拉取与 CSV 文件生成
//   - check() 负责权限校验，export() 仅做数据拉取与 CSV 生成
mod api_system;
mod api_user;
use crate::dao::WebExportTask;
use crate::dao::{AppSender, WebApp, WebError, WebRbac};
use lsys_core::app_core::AppCore;
use sqlx::{MySql, Pool};

use api_user::app_notify;
use api_user::app_request;
use api_user::file;
use api_user::login_history;
use api_user::mailer;
use api_user::mailer_tpl_body;
use api_user::rbac_audit;
use api_user::rbac_op;
use api_user::rbac_res;
use api_user::rbac_res_type;
use api_user::rbac_role;
use api_user::rbac_role_perm;
use api_user::rbac_role_user;
use api_user::role_user_available;
use api_user::smser;
use lsys_file::dao::FileDao;
use std::sync::Arc;

use api_user::app::{
    EXPORT_TYPE_USER_APP_LIST, EXPORT_TYPE_USER_PARENT_APP_LIST, EXPORT_TYPE_USER_SUB_APP_LIST,
    UserAppListExporter, UserParentAppListExporter, UserSubAppListExporter,
};
use app_notify::{AppNotifyListExporter, EXPORT_TYPE_APP_NOTIFY_LIST};
use app_request::{
    AppRequestListExporter, AppSubRequestListExporter, EXPORT_TYPE_USER_APP_REQUEST,
    EXPORT_TYPE_USER_SUB_REQUEST,
};
use file::{
    EXPORT_TYPE_USER_FILE_CHUNK, EXPORT_TYPE_USER_FILE_LIST, EXPORT_TYPE_USER_FILE_LOG,
    FileChunkExporter, FileListExporter, FileLogExporter,
};
use login_history::{EXPORT_TYPE_USER_LOGIN_HISTORY, UserLoginHistoryExporter};
use lsys_access::dao::AccessDao;
use lsys_user::dao::AccountDao;
use mailer::{
    EXPORT_TYPE_USER_MAILER_MESSAGE_LIST, EXPORT_TYPE_USER_MAILER_MESSAGE_LOG,
    EXPORT_TYPE_USER_MAILER_TPL_CONFIG, MailerMessageListExporter, MailerMessageLogExporter,
    MailerTplConfigExporter,
};
use mailer_tpl_body::{EXPORT_TYPE_USER_MAILER_TPL_BODY, MailerTplBodyExporter};
use rbac_audit::{
    EXPORT_TYPE_USER_RBAC_APP_AUDIT, EXPORT_TYPE_USER_RBAC_SYSTEM_AUDIT, RbacAuditExporter,
};
use rbac_op::{EXPORT_TYPE_USER_RBAC_APP_OP, RbacOpExporter};
use rbac_res::{EXPORT_TYPE_USER_RBAC_APP_RES, RbacResExporter};
use rbac_res_type::{
    EXPORT_TYPE_USER_RBAC_APP_RES_TYPE, EXPORT_TYPE_USER_RBAC_APP_RES_TYPE_OP, RbacResTypeExporter,
    RbacResTypeOpExporter,
};
use rbac_role::{
    EXPORT_TYPE_USER_RBAC_APP_ROLE, EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE, RbacRoleExporter,
};
use rbac_role_perm::{
    EXPORT_TYPE_USER_RBAC_APP_ROLE_PERM, EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_PERM,
    RbacRolePermExporter,
};
use rbac_role_user::{
    EXPORT_TYPE_USER_RBAC_APP_ROLE_USER, EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER,
    RbacRoleUserExporter,
};
use role_user_available::{
    EXPORT_TYPE_USER_APP_ROLE_USER_AVAILABLE, EXPORT_TYPE_USER_SYSTEM_ROLE_USER_AVAILABLE,
    RoleUserAvailableExporter,
};
use smser::{
    EXPORT_TYPE_USER_SMSER_MESSAGE_LIST, EXPORT_TYPE_USER_SMSER_MESSAGE_LOG,
    EXPORT_TYPE_USER_SMSER_TPL_CONFIG, SmserMessageListExporter, SmserMessageLogExporter,
    SmserTplConfigExporter,
};

use api_system::account_search::{EXPORT_TYPE_SYSTEM_ACCOUNT_SEARCH, SystemAccountSearchExporter};
use api_system::app_list::{
    EXPORT_TYPE_SYSTEM_APP_LIST, EXPORT_TYPE_SYSTEM_REQUEST_LIST, EXPORT_TYPE_SYSTEM_SUB_APP_LIST,
    SystemAppListExporter, SystemRequestListExporter, SystemSubAppListExporter,
};
use api_system::change_log::{EXPORT_TYPE_SYSTEM_CHANGE_LOG, SystemChangeLogExporter};
use api_system::login_history::{EXPORT_TYPE_SYSTEM_LOGIN_HISTORY, SystemLoginHistoryExporter};
use api_system::rbac_audit::{EXPORT_TYPE_SYSTEM_RBAC_AUDIT, SystemRbacAuditExporter};
use api_system::rbac_op::{EXPORT_TYPE_SYSTEM_RBAC_OP, SystemRbacOpExporter};
use api_system::rbac_res::{EXPORT_TYPE_SYSTEM_RBAC_RES, SystemRbacResExporter};
use api_system::rbac_res_type::{
    EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE, EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE_OP,
    SystemRbacResTypeExporter, SystemRbacResTypeOpExporter,
};
use api_system::rbac_role::{EXPORT_TYPE_SYSTEM_RBAC_ROLE, SystemRbacRoleExporter};
use api_system::rbac_role_perm::{EXPORT_TYPE_SYSTEM_RBAC_ROLE_PERM, SystemRbacRolePermExporter};
use api_system::rbac_role_user::{EXPORT_TYPE_SYSTEM_RBAC_ROLE_USER, SystemRbacRoleUserExporter};
use api_system::role_user_available::{
    EXPORT_TYPE_SYSTEM_ROLE_USER_AVAILABLE, SystemRoleUserAvailableExporter,
};
use lsys_logger::dao::ChangeLoggerDao;

#[allow(clippy::too_many_arguments)]
pub async fn register_exporters(
    db: Pool<MySql>,
    app_core: &AppCore,
    web_rbac: Arc<WebRbac>,
    account_dao: Arc<AccountDao>,
    access_dao: Arc<AccessDao>,
    web_app: Arc<WebApp>,
    app_sender: Arc<AppSender>,
    change_logger_dao: Arc<ChangeLoggerDao>,
    file_dao: Arc<FileDao>,
) -> Result<WebExportTask, WebError> {
    // 创建底层 ExportTask
    let export_task = lsys_file_manager::ExportTask::new(
        db,
        file_dao.clone(),
        change_logger_dao.clone(),
        app_core,
    );

    // 创建 WebExportTask 包装器
    let mut web_export_task = WebExportTask::new(Arc::new(export_task));

    // 登录历史（用户）
    web_export_task.register(
        EXPORT_TYPE_USER_LOGIN_HISTORY,
        Arc::new(UserLoginHistoryExporter {
            account_dao: account_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 文件列表 / 日志 / 分片
    web_export_task.register(
        EXPORT_TYPE_USER_FILE_LIST,
        Arc::new(FileListExporter {
            file_dao: file_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_FILE_LOG,
        Arc::new(FileLogExporter {
            file_dao: file_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_FILE_CHUNK,
        Arc::new(FileChunkExporter {
            file_dao,
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 应用列表（用户）
    web_export_task.register(
        EXPORT_TYPE_USER_APP_LIST,
        Arc::new(UserAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_PARENT_APP_LIST,
        Arc::new(UserParentAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_SUB_APP_LIST,
        Arc::new(UserSubAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 应用请求列表（用户）
    web_export_task.register(
        EXPORT_TYPE_USER_APP_REQUEST,
        Arc::new(AppRequestListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_SUB_REQUEST,
        Arc::new(AppSubRequestListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 应用通知
    web_export_task.register(
        EXPORT_TYPE_APP_NOTIFY_LIST,
        Arc::new(AppNotifyListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 操作
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_OP,
        Arc::new(RbacOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 角色（系统角色 / 应用角色）
    let rbac_role_exporter = Arc::new(RbacRoleExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
    });
    web_export_task.register(EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE, rbac_role_exporter)?;
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_ROLE,
        Arc::new(RbacRoleExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 角色用户（系统角色 / 应用角色）
    let rbac_role_user_exporter = Arc::new(RbacRoleUserExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
    });
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER,
        rbac_role_user_exporter,
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_ROLE_USER,
        Arc::new(RbacRoleUserExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 角色权限（系统角色 / 应用角色）
    let rbac_role_perm_exporter = Arc::new(RbacRolePermExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
    });
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_PERM,
        rbac_role_perm_exporter,
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_ROLE_PERM,
        Arc::new(RbacRolePermExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 资源
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_RES,
        Arc::new(RbacResExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 资源类型 / 资源类型操作
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_RES_TYPE,
        Arc::new(RbacResTypeExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_RES_TYPE_OP,
        Arc::new(RbacResTypeOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 审计（系统 / 应用）
    let rbac_audit_exporter = Arc::new(RbacAuditExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
    });
    web_export_task.register(EXPORT_TYPE_USER_RBAC_SYSTEM_AUDIT, rbac_audit_exporter)?;
    web_export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_AUDIT,
        Arc::new(RbacAuditExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 用户可用角色（用户系统角色 / 应用角色）
    let role_user_available_exporter = Arc::new(RoleUserAvailableExporter {
        access_dao: access_dao.clone(),
        web_rbac: web_rbac.clone(),
    });
    web_export_task.register(
        EXPORT_TYPE_USER_SYSTEM_ROLE_USER_AVAILABLE,
        role_user_available_exporter,
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_APP_ROLE_USER_AVAILABLE,
        Arc::new(RoleUserAvailableExporter {
            access_dao: access_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 邮件消息 / 日志 / 模板配置
    web_export_task.register(
        EXPORT_TYPE_USER_MAILER_MESSAGE_LIST,
        Arc::new(MailerMessageListExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_MAILER_MESSAGE_LOG,
        Arc::new(MailerMessageLogExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_MAILER_TPL_CONFIG,
        Arc::new(MailerTplConfigExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 邮件模板内容
    web_export_task.register(
        EXPORT_TYPE_USER_MAILER_TPL_BODY,
        Arc::new(MailerTplBodyExporter {
            tpl_dao: app_sender.tpl.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 短信消息 / 日志 / 模板配置
    web_export_task.register(
        EXPORT_TYPE_USER_SMSER_MESSAGE_LIST,
        Arc::new(SmserMessageListExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_SMSER_MESSAGE_LOG,
        Arc::new(SmserMessageLogExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_USER_SMSER_TPL_CONFIG,
        Arc::new(SmserTplConfigExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // ======== 系统管理端导出 ========

    // 系统应用列表
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_APP_LIST,
        Arc::new(SystemAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_SUB_APP_LIST,
        Arc::new(SystemSubAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_REQUEST_LIST,
        Arc::new(SystemRequestListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 角色
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_ROLE,
        Arc::new(SystemRbacRoleExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 资源
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_RES,
        Arc::new(SystemRbacResExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 资源类型 / 资源类型操作
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE,
        Arc::new(SystemRbacResTypeExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE_OP,
        Arc::new(SystemRbacResTypeOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 操作
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_OP,
        Arc::new(SystemRbacOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 角色权限
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_ROLE_PERM,
        Arc::new(SystemRbacRolePermExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 角色用户
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_ROLE_USER,
        Arc::new(SystemRbacRoleUserExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 审计
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_AUDIT,
        Arc::new(SystemRbacAuditExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统用户搜索
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_ACCOUNT_SEARCH,
        Arc::new(SystemAccountSearchExporter {
            account_dao: account_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统变更日志
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_CHANGE_LOG,
        Arc::new(SystemChangeLogExporter {
            change_logger_dao: change_logger_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统登录会话历史
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_LOGIN_HISTORY,
        Arc::new(SystemLoginHistoryExporter {
            access_dao: access_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统可用角色用户
    web_export_task.register(
        EXPORT_TYPE_SYSTEM_ROLE_USER_AVAILABLE,
        Arc::new(SystemRoleUserAvailableExporter {
            access_dao: access_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    Ok(web_export_task)
}
