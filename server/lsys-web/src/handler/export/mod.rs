// 内置 Exporter 实现集合
//
// 每个子模块对应一类列表接口的完整导出实现：
//   - 定义 `EXPORT_TYPE_XXX` 常量，注册到 `WebExportTask` 时使用
//   - 实现 `Exporter` trait，驱动实际数据拉取与 CSV 文件生成
//   - check() 负责权限校验，export() 仅做数据拉取与 CSV 生成
mod api_system;
mod api_user;
use crate::dao::export_task::WebExportTask;
use crate::dao::{AppSender, WebApp,  WebRbac};

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
use lsys_files::dao::FileDao;
use std::sync::Arc;

use api_user::app::{
    UserAppListExporter, UserParentAppListExporter, UserSubAppListExporter,
    EXPORT_TYPE_USER_APP_LIST, EXPORT_TYPE_USER_PARENT_APP_LIST, EXPORT_TYPE_USER_SUB_APP_LIST,
};
use app_notify::{AppNotifyListExporter, EXPORT_TYPE_APP_NOTIFY_LIST};
use app_request::{
    AppRequestListExporter, AppSubRequestListExporter, EXPORT_TYPE_USER_APP_REQUEST,
    EXPORT_TYPE_USER_SUB_REQUEST,
};
use file::{
    FileChunkExporter, FileListExporter, FileLogExporter, EXPORT_TYPE_USER_FILE_CHUNK,
    EXPORT_TYPE_USER_FILE_LIST, EXPORT_TYPE_USER_FILE_LOG,
};
use login_history::{UserLoginHistoryExporter, EXPORT_TYPE_USER_LOGIN_HISTORY};
use lsys_access::dao::AccessDao;
use lsys_user::dao::AccountDao;
use mailer::{
    MailerMessageListExporter, MailerMessageLogExporter, MailerTplConfigExporter,
    EXPORT_TYPE_USER_MAILER_MESSAGE_LIST, EXPORT_TYPE_USER_MAILER_MESSAGE_LOG,
    EXPORT_TYPE_USER_MAILER_TPL_CONFIG,
};
use mailer_tpl_body::{MailerTplBodyExporter, EXPORT_TYPE_USER_MAILER_TPL_BODY};
use rbac_audit::{
    RbacAuditExporter, EXPORT_TYPE_USER_RBAC_APP_AUDIT, EXPORT_TYPE_USER_RBAC_SYSTEM_AUDIT,
};
use rbac_op::{RbacOpExporter, EXPORT_TYPE_USER_RBAC_APP_OP};
use rbac_res::{RbacResExporter, EXPORT_TYPE_USER_RBAC_APP_RES};
use rbac_res_type::{
    RbacResTypeExporter, RbacResTypeOpExporter, EXPORT_TYPE_USER_RBAC_APP_RES_TYPE,
    EXPORT_TYPE_USER_RBAC_APP_RES_TYPE_OP,
};
use rbac_role::{
    RbacRoleExporter, EXPORT_TYPE_USER_RBAC_APP_ROLE, EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE,
};
use rbac_role_perm::{
    RbacRolePermExporter, EXPORT_TYPE_USER_RBAC_APP_ROLE_PERM,
    EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_PERM,
};
use rbac_role_user::{
    RbacRoleUserExporter, EXPORT_TYPE_USER_RBAC_APP_ROLE_USER,
    EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER,
};
use role_user_available::{
    RoleUserAvailableExporter, EXPORT_TYPE_USER_APP_ROLE_USER_AVAILABLE,
    EXPORT_TYPE_USER_SYSTEM_ROLE_USER_AVAILABLE,
};
use smser::{
    SmserMessageListExporter, SmserMessageLogExporter, SmserTplConfigExporter,
    EXPORT_TYPE_USER_SMSER_MESSAGE_LIST, EXPORT_TYPE_USER_SMSER_MESSAGE_LOG,
    EXPORT_TYPE_USER_SMSER_TPL_CONFIG,
};

use api_system::account_search::{SystemAccountSearchExporter, EXPORT_TYPE_SYSTEM_ACCOUNT_SEARCH};
use api_system::app_list::{
    SystemAppListExporter, SystemRequestListExporter, SystemSubAppListExporter,
    EXPORT_TYPE_SYSTEM_APP_LIST, EXPORT_TYPE_SYSTEM_REQUEST_LIST, EXPORT_TYPE_SYSTEM_SUB_APP_LIST,
};
use api_system::change_log::{SystemChangeLogExporter, EXPORT_TYPE_SYSTEM_CHANGE_LOG};
use api_system::login_history::{SystemLoginHistoryExporter, EXPORT_TYPE_SYSTEM_LOGIN_HISTORY};
use api_system::rbac_audit::{SystemRbacAuditExporter, EXPORT_TYPE_SYSTEM_RBAC_AUDIT};
use api_system::rbac_op::{SystemRbacOpExporter, EXPORT_TYPE_SYSTEM_RBAC_OP};
use api_system::rbac_res::{SystemRbacResExporter, EXPORT_TYPE_SYSTEM_RBAC_RES};
use api_system::rbac_res_type::{
    SystemRbacResTypeExporter, SystemRbacResTypeOpExporter, EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE,
    EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE_OP,
};
use api_system::rbac_role::{SystemRbacRoleExporter, EXPORT_TYPE_SYSTEM_RBAC_ROLE};
use api_system::rbac_role_perm::{SystemRbacRolePermExporter, EXPORT_TYPE_SYSTEM_RBAC_ROLE_PERM};
use api_system::rbac_role_user::{SystemRbacRoleUserExporter, EXPORT_TYPE_SYSTEM_RBAC_ROLE_USER};
use api_system::role_user_available::{
    SystemRoleUserAvailableExporter, EXPORT_TYPE_SYSTEM_ROLE_USER_AVAILABLE,
};
use lsys_logger::dao::ChangeLoggerDao;

#[allow(clippy::too_many_arguments)]
pub async fn register_exporters(
    export_task: &mut WebExportTask,
    file_dao: Arc<FileDao>,
    web_rbac: Arc<WebRbac>,
    account_dao: Arc<AccountDao>,
    access_dao: Arc<AccessDao>,
    web_app: Arc<WebApp>,
    app_sender: Arc<AppSender>,
    change_logger_dao: Arc<ChangeLoggerDao>,
) -> Result<(), String> {
   
    // 登录历史（用户）
    export_task.register(
        EXPORT_TYPE_USER_LOGIN_HISTORY,
        Box::new(UserLoginHistoryExporter {
            account_dao: account_dao.clone(),
        }),
    )?;

    // 文件列表 / 日志 / 分片
    export_task.register(
        EXPORT_TYPE_USER_FILE_LIST,
        Box::new(FileListExporter {
            file_dao: file_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_FILE_LOG,
        Box::new(FileLogExporter {
            file_dao: file_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_FILE_CHUNK,
        Box::new(FileChunkExporter {
            file_dao,
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // 应用列表（用户）
    export_task.register(
        EXPORT_TYPE_USER_APP_LIST,
        Box::new(UserAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_PARENT_APP_LIST,
        Box::new(UserParentAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_SUB_APP_LIST,
        Box::new(UserSubAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 应用请求列表（用户）
    export_task.register(
        EXPORT_TYPE_USER_APP_REQUEST,
        Box::new(AppRequestListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_SUB_REQUEST,
        Box::new(AppSubRequestListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 应用通知
    export_task.register(
        EXPORT_TYPE_APP_NOTIFY_LIST,
        Box::new(AppNotifyListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // RBAC 操作
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_OP,
        Box::new(RbacOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // RBAC 角色（系统角色 / 应用角色）
    let rbac_role_exporter = Box::new(RbacRoleExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
        web_app: web_app.clone(),
    });
    export_task.register(EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE, rbac_role_exporter)?;
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_ROLE,
        Box::new(RbacRoleExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // RBAC 角色用户（系统角色 / 应用角色）
    let rbac_role_user_exporter = Box::new(RbacRoleUserExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
        web_app: web_app.clone(),
    });
    export_task.register(
        EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER,
        rbac_role_user_exporter,
    )?;
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_ROLE_USER,
        Box::new(RbacRoleUserExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // RBAC 角色权限（系统角色 / 应用角色）
    let rbac_role_perm_exporter = Box::new(RbacRolePermExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
        web_app: web_app.clone(),
    });
    export_task.register(
        EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_PERM,
        rbac_role_perm_exporter,
    )?;
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_ROLE_PERM,
        Box::new(RbacRolePermExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // RBAC 资源
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_RES,
        Box::new(RbacResExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // RBAC 资源类型 / 资源类型操作
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_RES_TYPE,
        Box::new(RbacResTypeExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_RES_TYPE_OP,
        Box::new(RbacResTypeOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // RBAC 审计（系统 / 应用）
    let rbac_audit_exporter = Box::new(RbacAuditExporter {
        rbac_dao: web_rbac.rbac_dao.clone(),
        web_rbac: web_rbac.clone(),
        web_app: web_app.clone(),
    });
    export_task.register(EXPORT_TYPE_USER_RBAC_SYSTEM_AUDIT, rbac_audit_exporter)?;
    export_task.register(
        EXPORT_TYPE_USER_RBAC_APP_AUDIT,
        Box::new(RbacAuditExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // 用户可用角色（用户系统角色 / 应用角色）
    let role_user_available_exporter = Box::new(RoleUserAvailableExporter {
        access_dao: access_dao.clone(),
        web_rbac: web_rbac.clone(),
        web_app: web_app.clone(),
    });
    export_task.register(
        EXPORT_TYPE_USER_SYSTEM_ROLE_USER_AVAILABLE,
        role_user_available_exporter,
    )?;
    export_task.register(
        EXPORT_TYPE_USER_APP_ROLE_USER_AVAILABLE,
        Box::new(RoleUserAvailableExporter {
            access_dao: access_dao.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // 邮件消息 / 日志 / 模板配置
    export_task.register(
        EXPORT_TYPE_USER_MAILER_MESSAGE_LIST,
        Box::new(MailerMessageListExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_MAILER_MESSAGE_LOG,
        Box::new(MailerMessageLogExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_MAILER_TPL_CONFIG,
        Box::new(MailerTplConfigExporter {
            mailer_dao: app_sender.mailer.mailer_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 邮件模板内容
    export_task.register(
        EXPORT_TYPE_USER_MAILER_TPL_BODY,
        Box::new(MailerTplBodyExporter {
            tpl_dao: app_sender.tpl.clone(),
            web_rbac: web_rbac.clone(),
            web_app: web_app.clone(),
        }),
    )?;

    // 短信消息 / 日志 / 模板配置
    export_task.register(
        EXPORT_TYPE_USER_SMSER_MESSAGE_LIST,
        Box::new(SmserMessageListExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_SMSER_MESSAGE_LOG,
        Box::new(SmserMessageLogExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_USER_SMSER_TPL_CONFIG,
        Box::new(SmserTplConfigExporter {
            smser_dao: app_sender.smser.smser_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // ======== 系统管理端导出 ========

    // 系统应用列表
    export_task.register(
        EXPORT_TYPE_SYSTEM_APP_LIST,
        Box::new(SystemAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_SYSTEM_SUB_APP_LIST,
        Box::new(SystemSubAppListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_SYSTEM_REQUEST_LIST,
        Box::new(SystemRequestListExporter {
            app_dao: web_app.app_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 角色
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_ROLE,
        Box::new(SystemRbacRoleExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 资源
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_RES,
        Box::new(SystemRbacResExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 资源类型 / 资源类型操作
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE,
        Box::new(SystemRbacResTypeExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE_OP,
        Box::new(SystemRbacResTypeOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 操作
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_OP,
        Box::new(SystemRbacOpExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 角色权限
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_ROLE_PERM,
        Box::new(SystemRbacRolePermExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 角色用户
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_ROLE_USER,
        Box::new(SystemRbacRoleUserExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统 RBAC 审计
    export_task.register(
        EXPORT_TYPE_SYSTEM_RBAC_AUDIT,
        Box::new(SystemRbacAuditExporter {
            rbac_dao: web_rbac.rbac_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统用户搜索
    export_task.register(
        EXPORT_TYPE_SYSTEM_ACCOUNT_SEARCH,
        Box::new(SystemAccountSearchExporter {
            account_dao: account_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统变更日志
    export_task.register(
        EXPORT_TYPE_SYSTEM_CHANGE_LOG,
        Box::new(SystemChangeLogExporter {
            change_logger_dao: change_logger_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统登录会话历史
    export_task.register(
        EXPORT_TYPE_SYSTEM_LOGIN_HISTORY,
        Box::new(SystemLoginHistoryExporter {
            access_dao: access_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    // 系统可用角色用户
    export_task.register(
        EXPORT_TYPE_SYSTEM_ROLE_USER_AVAILABLE,
        Box::new(SystemRoleUserAvailableExporter {
            access_dao: access_dao.clone(),
            web_rbac: web_rbac.clone(),
        }),
    )?;

    Ok(())
}
