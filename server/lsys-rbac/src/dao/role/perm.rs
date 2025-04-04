use crate::{
    dao::result::{RbacError, RbacResult},
    model::{
        RbacOpModel, RbacOpResModel, RbacOpResStatus, RbacPermModel, RbacPermModelRef,
        RbacPermStatus, RbacResModel, RbacRoleModel, RbacRoleResRange,
    },
};
use lsys_core::db::{Insert, ModelTableName, SqlExpr, Update, WhereOption};
use lsys_core::{db_option_executor, model_option_set, sql_format};
use lsys_core::{fluent_message, now_time, RequestEnv};
use serde::Serialize;
use sqlx::{FromRow, Row, Transaction};

use super::{logger::LogRolePerm, RbacRole};
use lsys_core::db::SqlQuote;
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
        if !RbacRoleResRange::Exclude.eq(role.user_range)
            && !RbacRoleResRange::Include.eq(role.user_range)
        {
            return Err(RbacError::System(fluent_message!("rbac-res-perm-wrong",{
                "name":&role.role_name,
                "role_id":role.id,
                "range":role.user_range
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

        let op_res = sqlx::query_as::<_, (u64, String)>(&sql_format!(
            "select op_id,res_type from {} where status={} and ({})",
            RbacOpResModel::table_name(),
            RbacOpResStatus::Enable,
            SqlExpr(format!(
                "({})",
                perm_vec
                    .iter()
                    .map(|e| sql_format!(
                        "res_type={} and user_id={} and app_id={} and op_id={}",
                        e.res.res_type,
                        e.res.user_id,
                        e.res.app_id,
                        e.op.id
                    ))
                    .collect::<Vec<_>>()
                    .join(") or (")
            ))
        ))
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

        let perm_res = sqlx::query_as::<_, (u64, u64, u64)>(&sql_format!(
            "select id,res_id,op_id from {} where role_id={} and ({})",
            RbacPermModel::table_name(),
            role.id,
            SqlExpr(format!(
                "({})",
                perm_vec
                    .iter()
                    .map(|e| sql_format!("res_id={} and op_id={}", e.res.id, e.op.id))
                    .collect::<Vec<_>>()
                    .join(") or (")
            ))
        ))
        .fetch_all(&self.db)
        .await?;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let nowtime = now_time().unwrap_or_default();
        let mut add_item = vec![];
        for RolePerm { op, res } in perm_vec {
            let mut is_updata = false;
            for (itemid, res_id, op_id) in perm_res.iter() {
                if *res_id == res.id && *op_id == op.id {
                    let item = model_option_set!(RbacPermModelRef,{
                        role_id:role.id,
                        change_time:nowtime,
                        change_user_id:add_user_id,
                        status:(RbacPermStatus::Enable as i8),
                    });
                    if let Err(err) = Update::< RbacPermModel, _>::new(item)
                        .execute_by_where(
                            &WhereOption::Where(sql_format!("id={}", *itemid)),
                            &mut *db,
                        )
                        .await
                    {
                        db.rollback().await?;
                        return Err(err.into());
                    }
                    is_updata = true;
                }
            }
            if !is_updata {
                add_item.push(model_option_set!(RbacPermModelRef,{
                    op_id:op.id,
                    res_id:res.id,
                    role_id:role.id,
                    change_time:nowtime,
                    change_user_id:add_user_id,
                    status:(RbacPermStatus::Enable as i8),
                }));
            }
        }
        if !add_item.is_empty() {
            if let Err(err) = Insert::<RbacPermModel, _>::new_vec(add_item)
                .execute(&mut *db)
                .await
            {
                db.rollback().await?;
                return Err(err.into());
            }
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
        let ddata = model_option_set!(RbacPermModelRef,{
            change_user_id:del_user_id,
            change_time:time,
            status:(RbacPermStatus::Delete as i8),
        });
        let db = &self.db;
        let res = db_option_executor!(
            db,
            {
                Update::< RbacPermModel, _>::new(ddata)
                    .execute_by_where(
                        &lsys_core::db::WhereOption::Where(sql_format!(
                            "role_id ={} and ({})",
                            role.id,
                            SqlExpr(format!(
                                "({})",
                                perm_vec
                                    .iter()
                                    .map(|e| sql_format!(
                                        "res_id={} and op_id={}",
                                        e.res.id,
                                        e.op.id
                                    ))
                                    .collect::<Vec<_>>()
                                    .join(") or (")
                            ))
                        )),
                        db.as_executor(),
                    )
                    .await?
            },
            transaction,
            db
        );

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
        let mut perm_id = 0;
        loop {
            let role_data=sqlx::query(&sql_format!(
                "select role.*,perm.op_id,perm.id as perm_id from {} as role join {} as perm on role.id=perm.role_id where perm.res_id={} and perm.op_id in ({}) and perm.id>{} order by perm.id asc limit 100 ",
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                res.id,
                op_data,
                perm_id
            ))
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
            let role_data=sqlx::query(&sql_format!(
                "select role.*,perm.op_id,perm.id as perm_id from {} as role join {} as perm on role.id=perm.role_id where perm.res_id={} and perm.id>{} order by perm.id asc limit 100",
                RbacRoleModel::table_name(),
                RbacPermModel::table_name(),
                res.id,
                perm_id
            ))
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
            let change = lsys_core::model_option_set!(RbacPermModelRef,{
                status:status,
                change_user_id:delete_user_id,
                change_time:time,
            });
            let tmp = Update::< RbacPermModel, _>::new(change)
                .execute_by_where(
                    &WhereOption::Where(sql_format!(
                        "role_id={} and op_id={op_id} and res_id={}",
                        role.id,
                        res.id
                    )),
                    &mut *db,
                )
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
