// 用户 APP RBAC 角色列表导出（角色 × 用户 × 资源权限 三维笛卡尔积）
//
//   AppRoleDataExporter — 对指定 APP 的角色列表展开：
//     1. 循环角色列表
//     2. 仅当 user_range == Custom 时，查询关联用户列表（Session 角色无固定用户）
//     3. 仅当 res_range == Include 或 Exclude 时，查询关联资源/操作列表
//        （Any 表示允许所有资源，无需列出 perm 记录）
//     4. 输出 (角色, 用户, 资源操作) 三维笛卡尔积的每条记录
//
//   CSV 列: role_id, role_key, role_name, user_range, res_range,
//           user_id, user_timeout,
//           op_key, op_name, res_type, res_data, res_name
//   AppRoleDataExportCheck — 权限检查器

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, RoleDataAttrParam, RoleDataParam};
use lsys_rbac::model::{RbacRoleResRange, RbacRoleUserRange};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_APP_ROLE_DATA: &str = "app_role_data";

/// APP 角色数据权限检查器
pub struct AppRoleDataExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for AppRoleDataExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserRbacView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

/// APP 角色数据导出器（三维笛卡尔积）
pub struct AppRoleDataExporter {
    pub rbac_dao: Arc<RbacDao>,
}

impl Exporter<crate::dao::WebError> for AppRoleDataExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
        lang: Option<String>,
        fluent_mgr: Arc<lsys_core::fluents::FluentMgr>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let user_id = record.user_id;
            let app_id = params["app_id"].as_u64();
            let role_key = params["role_key"].as_str();
            let role_name = params["role_name"].as_str();

            let role_param = RoleDataParam {
                user_id,
                app_id,
                ids: None,
                user_range: None,
                res_range: None,
                role_key,
                role_name,
            };
            let role_attr = RoleDataAttrParam::default();

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_APP_ROLE_DATA,
                    "role_id",
                    "role_key",
                    "role_name",
                    "user_range",
                    "res_range",
                    "user_id",
                    "user_timeout",
                    "op_key",
                    "op_name",
                    "res_type",
                    "res_data",
                    "res_name",
                ))
                .await?;

            let total = self
                .rbac_dao
                .role
                .role_count(&role_param)
                .await? as u64;

            if total == 0 {
                return w.finish().await.map_err(Into::into);
            }

            let batch = 50u64;
            let mut offset = 0u64;
            loop {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(offset, batch)));
                let roles = self
                    .rbac_dao
                    .role
                    .role_info(&role_param, &role_attr, &page)
                    .await?;

                if roles.is_empty() {
                    break;
                }

                for (role, _) in &roles {
                    // 判断 user_range：仅 Custom 类型才有固定关联用户列表
                    let is_custom_user =
                        RbacRoleUserRange::Custom.eq(role.user_range);

                    // 判断 res_range：仅 Include / Exclude 才有 perm 记录
                    let has_perm_list = RbacRoleResRange::Include.eq(role.res_range)
                        || RbacRoleResRange::Exclude.eq(role.res_range);

                    // 查询关联用户（仅 Custom 用户范围才查）
                    let users = if is_custom_user {
                        self.rbac_dao
                            .role
                            .role_user_data(
                                role,
                                false,
                                &OffsetPageParam::new(Some(OffsetPageValue::new(
                                    0,
                                    u64::MAX / 2,
                                ))),
                            )
                            .await?
                    } else {
                        vec![]
                    };

                    // 查询关联权限（仅 Include / Exclude 资源范围才查）
                    let perms = if has_perm_list {
                        let perm_total =
                            self.rbac_dao.role.role_perm_count(role).await? as u64;
                        if perm_total > 0 {
                            self.rbac_dao
                                .role
                                .role_perm_data(
                                    role,
                                    &OffsetPageParam::new(Some(OffsetPageValue::new(
                                        0,
                                        perm_total,
                                    ))),
                                )
                                .await?
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    };

                    // ── 输出笛卡尔积 ──────────────────────────────────────────
                    match (users.is_empty(), perms.is_empty()) {
                        // 两者均无数据 → 输出一行仅含角色信息的记录
                        (true, true) => {
                            w.write_batch(vec![(
                                role.id,
                                role.role_key.clone(),
                                role.role_name.clone(),
                                role.user_range,
                                role.res_range,
                                0u64,
                                0u64,
                                String::new(),
                                String::new(),
                                String::new(),
                                String::new(),
                                String::new(),
                            )])
                            .await?;
                        }
                        // 有用户无权限（user_range=Custom 且 res_range=Any 或 Include/Exclude 但无具体 perm）
                        (false, true) => {
                            let rows: Vec<_> = users
                                .iter()
                                .map(|u| {
                                    (
                                        role.id,
                                        role.role_key.clone(),
                                        role.role_name.clone(),
                                        role.user_range,
                                        role.res_range,
                                        u.user_id,
                                        u.timeout,
                                        String::new(),
                                        String::new(),
                                        String::new(),
                                        String::new(),
                                        String::new(),
                                    )
                                })
                                .collect();
                            w.write_batch(rows).await?;
                        }
                        // 无用户有权限（user_range=Session 且 res_range=Include/Exclude）
                        (true, false) => {
                            let rows: Vec<_> = perms
                                .iter()
                                .map(|p| {
                                    (
                                        role.id,
                                        role.role_key.clone(),
                                        role.role_name.clone(),
                                        role.user_range,
                                        role.res_range,
                                        0u64,
                                        0u64,
                                        p.op_key.clone(),
                                        p.op_name.clone(),
                                        p.res_type.clone(),
                                        p.res_data.clone(),
                                        p.res_name.clone(),
                                    )
                                })
                                .collect();
                            w.write_batch(rows).await?;
                        }
                        // 有用户 × 有权限 → 完整三维笛卡尔积
                        (false, false) => {
                            let mut rows = Vec::with_capacity(users.len() * perms.len());
                            for u in &users {
                                for p in &perms {
                                    rows.push((
                                        role.id,
                                        role.role_key.clone(),
                                        role.role_name.clone(),
                                        role.user_range,
                                        role.res_range,
                                        u.user_id,
                                        u.timeout,
                                        p.op_key.clone(),
                                        p.op_name.clone(),
                                        p.res_type.clone(),
                                        p.res_data.clone(),
                                        p.res_name.clone(),
                                    ));
                                }
                            }
                            w.write_batch(rows).await?;
                        }
                    }
                }

                offset += roles.len() as u64;
                if offset >= total {
                    break;
                }
            }

            w.finish().await.map_err(Into::into)
        })
    }
}

//
//   AppRoleDataExporter — 对指定 APP 的角色列表展开：
