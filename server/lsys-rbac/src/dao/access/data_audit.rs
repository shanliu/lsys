use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::{sql_format, string_clear, STRING_CLEAR_FORMAT};
use lsys_core::{LimitParam, StringClear};

use crate::{
    dao::result::RbacResult,
    model::{RbacAuditDetailModel, RbacAuditModel},
};

use super::RbacAccess;
use lsys_core::db::SqlQuote;

//查询授权审计日志

pub struct AuditDataParam<'t> {
    pub user_id: Option<u64>,
    pub user_app_id: Option<u64>,
    pub user_ip: Option<&'t str>,
    pub device_id: Option<&'t str>,
    pub request_id: Option<&'t str>,
    pub res_data: Option<(u64, Option<u64>)>, //资源ID,资源关联操作ID
}

impl RbacAccess {
    fn audit_sql(&self, audit_param: &AuditDataParam<'_>) -> Option<Vec<String>> {
        let mut where_sql = vec![];
        if let Some(val) = audit_param.user_id {
            where_sql.push(sql_format!("  user_id = {}", val));
        }
        if let Some(val) = audit_param.user_app_id {
            where_sql.push(sql_format!("  user_app_id = {}", val));
        }
        if let Some(val) = audit_param.user_ip {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(47));

            where_sql.push(sql_format!("  user_ip = {}", val));
        }
        if let Some(val) = audit_param.device_id {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(65));

            where_sql.push(sql_format!("  device_id = {}", val));
        }
        if let Some(val) = audit_param.request_id {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(65));

            where_sql.push(sql_format!("  request_id = {}", val));
        }
        if let Some(val) = audit_param.res_data {
            where_sql.push(sql_format!(
                "  id in (select rbac_audit_id from {} where res_id={} {})",
                RbacAuditDetailModel::table_name(),
                val.0,
                match val.1 {
                    Some(op_id) => SqlExpr(sql_format!(" and op_id={}", op_id)),
                    None => SqlExpr("".to_string()),
                }
            ));
        }
        Some(where_sql)
    }
    /// 获取指定条件的角色数量
    pub async fn audit_count(&self, audit_param: &AuditDataParam<'_>) -> RbacResult<i64> {
        match self.audit_sql(audit_param) {
            Some(where_sql) => {
                let mut sql = sql_format!(
                    "select count(*) as total from {} ",
                    RbacAuditModel::table_name(),
                );
                if !where_sql.is_empty() {
                    sql += format!(" where {}", where_sql.join(" and ")).as_str();
                }
                let query = sqlx::query_scalar::<_, i64>(&sql);
                let res = query.fetch_one(&self.db).await?;
                Ok(res)
            }
            None => Ok(0),
        }
    }
    /// 获取指定用户和ID的列表
    pub async fn audit_data(
        &self,
        res_param: &AuditDataParam<'_>,
        limit: Option<&LimitParam>,
    ) -> RbacResult<(
        Vec<(RbacAuditModel, Vec<RbacAuditDetailModel>)>,
        Option<u64>,
    )> {
        match self.audit_sql(res_param) {
            Some(sqlwhere) => {
                let sql = format!(
                    "select * from {} {}",
                    RbacAuditModel::table_name(),
                    if let Some(page) = limit {
                        let page_where = page.where_sql(
                            "id",
                            if sqlwhere.is_empty() {
                                None
                            } else {
                                Some("and")
                            },
                        );
                        format!(
                            "{} {} {} order by {} {} ",
                            if !sqlwhere.is_empty() || !page_where.is_empty() {
                                "where "
                            } else {
                                ""
                            },
                            sqlwhere.join(" and "),
                            page_where,
                            page.order_sql("id"),
                            page.limit_sql(),
                        )
                    } else {
                        format!(
                            "{} {}  order by id desc",
                            if !sqlwhere.is_empty() { "where " } else { "" },
                            sqlwhere.join(" and ")
                        )
                    }
                );
                let mut row = sqlx::query_as::<_, RbacAuditModel>(&sql)
                    .fetch_all(&self.db)
                    .await?;
                let next = limit
                    .as_ref()
                    .map(|page| page.tidy(&mut row))
                    .unwrap_or_default();

                let mut out_data = Vec::with_capacity(row.len());
                if !row.is_empty() {
                    let mut detail_row = sqlx::query_as::<_, RbacAuditDetailModel>(&sql_format!(
                        "select * from {} where rbac_audit_id in ({})",
                        RbacAuditDetailModel::table_name(),
                        row.iter().map(|e| e.id).collect::<Vec<_>>()
                    ))
                    .fetch_all(&self.db)
                    .await?;
                    for tmp in row {
                        let mut dtmp = vec![];
                        let mut otmp = vec![];
                        for itmp in detail_row {
                            if itmp.rbac_audit_id == tmp.id {
                                otmp.push(itmp);
                            } else {
                                dtmp.push(itmp);
                            }
                        }
                        out_data.push((tmp, otmp));
                        detail_row = dtmp;
                    }
                }

                Ok((out_data, next.map(|t| t.id)))
            }
            None => Ok((vec![], None)),
        }
    }
}
