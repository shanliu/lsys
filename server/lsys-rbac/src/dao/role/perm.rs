use crate::{
    dao::result::{RbacError, RbacResult},
    model::{
        RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacPermModel, RbacPermStatus, RbacResModel,
        RbacRoleModel, RbacRoleResRange,
    },
};
use lsys_core::db::{BatchInsert, Insert, QueryBuilderExt, TableMeta, Update};
use lsys_core::utils::{now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT};
use lsys_core::db::OptionTxExecutor;
use lsys_core::fluent_message;
use serde::Serialize;
use sqlx::{FromRow, MySql, QueryBuilder, Row, Transaction};

use super::{logger::LogRolePerm, RbacRole};
use sqlx::Acquire;

//角色对应授权的实现

#[derive(Clone, Debug, Serialize)]
pub struct RolePerm<'t> {
    pub op: &'t RbacOpModel,
    pub res: &'t RbacResModel,
}

impl RbacRole {
    //添加权限
    pub async fn add_perm(
        &self,
        role: &RbacRoleModel,
        perm_vec: &[RolePerm<'_>],
        add_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        if perm_vec.is_empty() {
            return Ok(());
        }
        if !RbacRoleResRange::Exclude.eq(role.res_range)
            && !RbacRoleResRange::Include.eq(role.res_range)
        {
            return Err(RbacError::System(fluent_message!("rbac-res-perm-wrong",{
                "name":&role.role_name,
                "role_id":role.id,
                "range":role.res_range
            })));
        }

        for perm in perm_vec {
            if perm.op.user_id != perm.res.user_id || perm.op.app_id != perm.res.app_id {
                return Err(RbacError::System(fluent_message!("rbac-role-bad-op-user",{
                    "res":&perm.res.res_name,
                    "op":&perm.op.op_name,
                    "op_user_id":perm.op.user_id,
                })));
            }
        }

        if role.user_id > 0 {
            //系统内置用户
            if role.app_id > 0 {
                //非系统用户,只能限定APP相同
                for perm in perm_vec {
                    if perm.res.user_id != role.user_id || perm.res.app_id != role.app_id {
                        return Err(RbacError::System(
                            fluent_message!("rbac-role-bad-perm-user",{
                                "res":&perm.res.res_name,
                                "op":&perm.op.op_name,
                                "user_id":role.user_id,
                            }),
                        ));
                    }
                }
            } else {
                for perm in perm_vec {
                    if perm.res.user_id != role.user_id {
                        return Err(RbacError::System(
                            fluent_message!("rbac-role-bad-perm-user",{
                                "res":&perm.res.res_name,
                                "op":&perm.op.op_name,
                                "user_id":role.user_id,
                            }),
                        ));
                    }
                }
            }
        }

        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select op_id,res_type from {}",
            RbacOpResModel::table_name(),
        ));
        qb.push_where().field_eq("status", RbacOpResStatus::Enable as i8).push_and().push("((");
        for (i, e) in perm_vec.iter().enumerate() {
            if i > 0 {
                qb.push(") or (");
            }
            let res_type = string_clear(
                &e.res.res_type,
                StringClear::Option(STRING_CLEAR_FORMAT),
                Some(33),
            );
            qb.field_eq("res_type", res_type);
            qb.push_and().field_eq("user_id", e.res.user_id);
            qb.push_and().field_eq("app_id", e.res.app_id);
            qb.push_and().field_eq("op_id", e.op.id);
        }
        qb.push("))");
        let op_res = qb.build_query_as::<(u64, String)>()
        .fetch_all(&self.db)
        .await?;

        for perm in perm_vec {
            if !op_res
                .iter()
                .any(|(opid, res_type)| *opid == perm.op.id && perm.res.res_type == *res_type)
            {
                return Err(RbacError::System(fluent_message!("rbac-role-bad-res-op",{
                    "res":&perm.res.res_name,
                    "op":&perm.op.op_name,
                })));
            }
        }

        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select id,res_id,op_id from {}",
            RbacPermModel::table_name(),
        ));
        qb.push_where().field_eq("role_id", role.id);
        qb.push_and().push("((");
        for (idx, perm) in perm_vec.iter().enumerate() {
            if idx > 0 {
                qb.push(") or (");
            }
            qb.field_eq("res_id", perm.res.id);
            qb.push_and().field_eq("op_id", perm.op.id);
        }
        qb.push("))");
        let perm_res = qb.build_query_as::<(u64, u64, u64)>()
        .fetch_all(&self.db)
        .await?;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let nowtime = now_time().unwrap_or_default();
        let mut batch = BatchInsert::<_,RbacPermModel>::new();
        for RolePerm { op, res } in perm_vec {
            let mut is_updata = false;
            for (itemid, res_id, op_id) in perm_res.iter() {
                if *res_id == res.id && *op_id == op.id {
                    if let Err(err) = Update::<_,RbacPermModel>::new()
                        .set(RbacPermModel::ROLE_ID, role.id)
                        .set(RbacPermModel::CHANGE_TIME, nowtime)
                        .set(RbacPermModel::CHANGE_USER_ID, add_user_id)
                        .set(RbacPermModel::STATUS, RbacPermStatus::Enable as i8)
                        .execute(&mut *db, |qb| {
                            qb.push_where().field_eq("id", *itemid);
                        })
                        .await
                    {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                    is_updata = true;
                }
            }
            if !is_updata {
                batch = batch.push(
                    Insert::<_,RbacPermModel>::new()
                        .set(RbacPermModel::OP_ID, op.id)
                        .set(RbacPermModel::RES_ID, res.id)
                        .set(RbacPermModel::ROLE_ID, role.id)
                        .set(RbacPermModel::CHANGE_TIME, nowtime)
                        .set(RbacPermModel::CHANGE_USER_ID, add_user_id)
                        .set(RbacPermModel::STATUS, RbacPermStatus::Enable as i8),
                );
            }
        }
        if !batch.is_empty()
            && let Err(err) = batch.execute(&mut *db).await {
                db.rollback().await?;
                return Err(err.into());
            }
        db.commit().await?;

        let res_op_data = perm_vec
            .iter()
            .map(|p| (p.res.id, p.op.id))
            .collect::<Vec<_>>();

        self.cache()
            .clear_access(role, Some(&res_op_data), Some(&[]))
            .await;

        self.logger
            .add(
                &LogRolePerm {
                    action: "add",
                    user_id: role.user_id,
                    name: &role.role_name,
                    add_user: Some(res_op_data),
                    del_user: None,
                },
                Some(role.id),
                Some(add_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    //删除权限
    pub async fn del_perm(
        &self,
        role: &RbacRoleModel,
        perm_vec: &[RolePerm<'_>],
        del_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        if perm_vec.is_empty() {
            return Ok(0);
        }
        let time = now_time().unwrap_or_default();
        
        let res = Update::<_,RbacPermModel>::new()
            .set(RbacPermModel::CHANGE_USER_ID, del_user_id)
            .set(RbacPermModel::CHANGE_TIME, time)
            .set(RbacPermModel::STATUS, RbacPermStatus::Delete as i8)
            .execute(
                OptionTxExecutor::new(transaction, &self.db),
                |qb| {
                    qb.push_where().field_eq("role_id", role.id).push_and().push("(");
                    for (idx, perm) in perm_vec.iter().enumerate() {
                        if idx > 0 {
                            qb.push(") OR (");
                        }
                        qb.field_eq("res_id", perm.res.id).push_and().field_eq("op_id", perm.op.id);
                    }
                    qb.push(")");
                },
            )
            .await?;

        let res_op_data = perm_vec
            .iter()
            .map(|p| (p.res.id, p.op.id))
            .collect::<Vec<_>>();

        self.cache()
            .clear_access(role, Some(&res_op_data), Some(&[]))
            .await;

        self.logger
            .add(
                &LogRolePerm {
                    action: "del",
                    name: role.role_name.as_str(),
                    add_user: Some(res_op_data),
                    del_user: None,
                    user_id: role.user_id,
                },
                Some(role.id),
                Some(del_user_id),
                None,
                env_data,
            )
            .await;
        Ok(res.rows_affected())
    }
    //从所有的角色关系中移除指定资源的指定操作数据
    pub(crate) async fn role_remove_perm(
        &self,
        res: &RbacResModel,
        op_data: &[u64],
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        match self
            ._role_remove_perm(res, op_data, delete_user_id, &mut db, env_data)
            .await
        {
            Ok(_) => {
                db.commit().await?;
                Ok(())
            }
            Err(err) => {
                db.rollback().await?;
                Err(err)
            }
        }
    }
    async fn _role_remove_perm(
        &self,
        res: &RbacResModel,
        op_data: &[u64],
        delete_user_id: u64,
        db: &mut Transaction<'_, sqlx::MySql>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        if op_data.is_empty() {
            return Ok(());
        }
        let mut perm_id = 0;
        loop {
            let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select role.*,perm.op_id,perm.id as perm_id 
                    from {} as role join {} as perm on role.id=perm.role_id",
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
            ));
            qb.push_where().field_eq("perm.res_id", res.id);
            qb.push_and().field_in_copied("perm.op_id", op_data);
            qb.push_and().field_gt("perm.id", perm_id).push(" order by perm.id asc limit 100 ");
            let role_data=qb.build()
            .try_map(
                |row: sqlx::mysql::MySqlRow| match RbacRoleModel::from_row(&row) {
                    Ok(role) => {
                        let op_id = row.try_get::<u64, &str>("op_id").unwrap_or_default();
                        perm_id=row.try_get::<u64, &str>("perm_id").unwrap_or(u64::MAX);
                        Ok((op_id,role))
                    }
                    Err(err) => Err(err),
                },
            )
            .fetch_all(&self.db)
            .await?;
            if role_data.is_empty() {
                break;
            }
            self.role_remove_perm_from_role_data(
                res,
                &role_data,
                delete_user_id,
                Some(db),
                env_data,
            )
            .await?;
        }
        Ok(())
    }
    //从所有的角色关系中移除指定的资源
    pub(crate) async fn role_remove_all_perm(
        &self,
        res: &RbacResModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        let mut perm_id = 0;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        loop {
            let role_data=sqlx::query(&format!(
                "select role.*,perm.op_id,perm.id as perm_id from {} as role join {} as perm on role.id=perm.role_id where perm.res_id=? and perm.id>? order by perm.id asc limit 100",
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
            ))
            .bind(res.id)
            .bind(perm_id)
            .try_map(
                |row: sqlx::mysql::MySqlRow| match RbacRoleModel::from_row(&row) {
                    Ok(role) => {
                        let op_id = row.try_get::<u64, &str>("op_id").unwrap_or_default();
                        perm_id=row.try_get::<u64, &str>("perm_id").unwrap_or(u64::MAX);
                        Ok((op_id,role))
                    }
                    Err(err) => Err(err),
                },
            )
            .fetch_all(&self.db)
            .await?;
            if role_data.is_empty() {
                break;
            }
            if let Err(err) = self
                .role_remove_perm_from_role_data(
                    res,
                    &role_data,
                    delete_user_id,
                    Some(&mut db),
                    env_data,
                )
                .await
            {
                db.rollback().await?;
                return Err(err);
            };
        }
        db.commit().await?;
        Ok(())
    }
    //根据角色跟权限移除数据
    async fn role_remove_perm_from_role_data(
        &self,
        res: &RbacResModel,
        role_data: &[(u64, RbacRoleModel)],
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let status = RbacPermStatus::Delete as i8;
        let time = now_time().unwrap_or_default();
        for (op_id, role) in role_data {
            let role_id = role.id;
            let op_id_val = *op_id;
            let tmp = Update::<_,RbacPermModel>::new()
                .set(RbacPermModel::STATUS, status)
                .set(RbacPermModel::CHANGE_USER_ID, delete_user_id)
                .set(RbacPermModel::CHANGE_TIME, time)
                .execute(&mut *db, |qb| {
                    qb.push_where().field_eq("role_id", role_id);
                    qb.push_and().field_eq("op_id", op_id_val);
                    qb.push_and().field_eq("res_id", res.id);
                })
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }
        db.commit().await?;
        for (op_id, role) in role_data {
            self.cache()
                .clear_access(role, Some(&[(res.id, *op_id)]), Some(&[]))
                .await;
            self.logger
                .add(
                    &LogRolePerm {
                        action: "del",
                        name: role.role_name.as_str(),
                        add_user: Some(vec![(res.id, *op_id)]),
                        del_user: None,
                        user_id: role.user_id,
                    },
                    Some(role.id),
                    Some(delete_user_id),
                    None,
                    env_data,
                )
                .await;
        }

        Ok(())
    }
}
