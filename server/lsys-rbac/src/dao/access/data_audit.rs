use lsys_core::db::{
    CursorPageData, CursorPageParam, QueryBuilderExt, TableMeta, TotalParam, TotalRow, WhereClause,
};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, string_clear};

use crate::{
    dao::result::RbacResult,
    model::{RbacAuditDetailModel, RbacAuditModel},
};

use super::RbacAccess;
use sqlx::{MySql, QueryBuilder};

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
    fn audit_push_where(wb: &mut WhereClause<'_, '_, MySql>, audit_param: &AuditDataParam<'_>) {
        if let Some(val) = audit_param.user_id {
            wb.and().field_eq("user_id", val);
        }
        if let Some(val) = audit_param.user_app_id {
            wb.and().field_eq("user_app_id", val);
        }
        if let Some(val) = audit_param.user_ip {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(47));
            wb.and().field_eq("user_ip", val);
        }
        if let Some(val) = audit_param.device_id {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(65));
            wb.and().field_eq("device_id", val);
        }
        if let Some(val) = audit_param.request_id {
            let val = string_clear(val, StringClear::Option(STRING_CLEAR_FORMAT), Some(65));
            wb.and().field_eq("request_id", val);
        }
        if let Some(val) = audit_param.res_data {
            let qb = wb.and();
            qb.push(format!(
                " id in (select rbac_audit_id from {}",
                RbacAuditDetailModel::table_name(),
            ));
            qb.push_where().field_eq("res_id", val.0);
            if let Some(op_id) = val.1 {
                qb.push_and().field_eq("op_id", op_id);
            }
            qb.push(")");
        }
    }
    /// 获取指定条件的角色数量
    pub async fn audit_count(
        &self,
        audit_param: &AuditDataParam<'_>,
        total_param: &TotalParam,
    ) -> RbacResult<TotalRow> {
        let query = total_param.total_count_query();
        let mut qb = if query.is_threshold_mode() {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from (select 1 from {}",
                RbacAuditModel::table_name(),
            ));
            {
                let mut wb = WhereClause::new(&mut qb);
                Self::audit_push_where(&mut wb, audit_param);
            }
            query.push_limit(&mut qb);
            qb.push(") as t");
            qb
        } else {
            let mut qb = QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from {}",
                RbacAuditModel::table_name(),
            ));
            {
                let mut wb = WhereClause::new(&mut qb);
                Self::audit_push_where(&mut wb, audit_param);
            }
            qb
        };
        let count = qb.build_query_scalar::<i64>().fetch_one(&self.db).await? as u64;
        Ok(query.finalize(count))
    }
    /// 获取指定用户和ID的列表
    pub async fn audit_data(
        &self,
        res_param: &AuditDataParam<'_>,
        limit: &CursorPageParam<u64>,
    ) -> RbacResult<(
        Vec<(RbacAuditModel, Vec<RbacAuditDetailModel>)>,
        CursorPageData<u64>,
    )> {
        let query_limit = limit.page_query("id");
        let mut qb =
            QueryBuilder::<MySql>::new(format!("select * from {}", RbacAuditModel::table_name(),));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::audit_push_where(&mut wb, res_param);
        }
        if query_limit.has_cursor() {
            qb.push_and();
            query_limit.push_where(&mut qb);
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);
        let mut row = qb
            .build_query_as::<RbacAuditModel>()
            .fetch_all(&self.db)
            .await?;
        let next = query_limit.finalize(&mut row, |c, d| *d == c.id, |c| c.id);

        let mut out_data = Vec::with_capacity(row.len());
        if !row.is_empty() {
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                RbacAuditDetailModel::table_name(),
            ));
            qb.push_where()
                .field_in("rbac_audit_id", row.iter().map(|e| e.id));
            let mut detail_row = qb
                .build_query_as::<RbacAuditDetailModel>()
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

        Ok((out_data, next))
    }
}
