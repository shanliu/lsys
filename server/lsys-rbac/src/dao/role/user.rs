//RBAC中角色相关实现

use lsys_core::{now_time, RequestEnv};

use serde::Serialize;

use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update, WhereOption};
use lsys_core::{db_option_executor, model_option_set, sql_format};
use sqlx::Transaction;

use super::{logger::LogRoleUser, RbacRole};
use crate::dao::role::fluent_message;
use crate::{
    dao::result::{RbacError, RbacResult},
    model::{
        RbacRoleModel, RbacRoleUserModel, RbacRoleUserModelRef, RbacRoleUserRange,
        RbacRoleUserStatus,
    },
};
use sqlx::Acquire;

//角色对应用户的实现

#[derive(Clone, Debug, Serialize)]
pub struct RoleAddUser {
    pub user_id: u64,
    pub timeout: u64, //换成时间不超过此值，查询时要有此值
}
impl RbacRole {
    pub async fn add_user(
        &self,
        role: &RbacRoleModel,
        user_vec: &[RoleAddUser],
        add_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        if user_vec.is_empty() {
            return Ok(());
        }
        if !RbacRoleUserRange::Custom.eq(role.user_range) {
            return Err(RbacError::System(
                fluent_message!("rbac-res-op-user-wrong",{
                    "name":&role.role_name,
                    "role_id":role.id,
                    "range":role.user_range
                }),
            ));
        }

        let user_id_vec = user_vec.iter().map(|e| e.user_id).collect::<Vec<_>>();

        let user_res = sqlx::query_as::<_, (u64, u64)>(&sql_format!(
            "select id,user_id from {} where user_id in ({}) and role_id={} ",
            RbacRoleUserModel::table_name(),
            user_id_vec,
            role.id,
        ))
        .fetch_all(&self.db)
        .await?;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let nowtime = now_time().unwrap_or_default();
        let mut add_item = vec![];
        for RoleAddUser { user_id, timeout } in user_vec.iter() {
            let mut is_updata = false;
            for (itemid, uid) in user_res.iter() {
                if uid == user_id {
                    let item = model_option_set!(RbacRoleUserModelRef,{
                        role_id:role.id,
                        change_time:nowtime,
                        change_user_id:add_user_id,
                        status:(RbacRoleUserStatus::Enable as i8),
                    });
                    if let Err(err) = Update::< RbacRoleUserModel, _>::new(item)
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
                add_item.push(model_option_set!(RbacRoleUserModelRef,{
                    user_id:user_id,
                    timeout:timeout,
                    role_id:role.id,
                    change_time:nowtime,
                    change_user_id:add_user_id,
                    status:(RbacRoleUserStatus::Enable as i8),
                }));
            }
        }
        if !add_item.is_empty() {
            if let Err(err) = Insert::<RbacRoleUserModel, _>::new_vec(add_item)
                .execute(&mut *db)
                .await
            {
                db.rollback().await?;
                return Err(err.into());
            }
        }
        db.commit().await?;

        self.cache()
            .clear_access(role, Some(&[]), Some(&user_id_vec))
            .await;

        self.logger
            .add(
                &LogRoleUser {
                    action: "add",
                    user_id: role.user_id,
                    name: role.role_name.as_str(),
                    add_user: Some(user_vec.to_owned()),
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
    /// 角色删除用户
    pub async fn del_user(
        &self,
        role: &RbacRoleModel,
        user_id_vec: &[u64],
        del_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        if user_id_vec.is_empty() {
            return Ok(0);
        }
        let time = now_time().unwrap_or_default();
        let ddata = model_option_set!(RbacRoleUserModelRef,{
            change_user_id:del_user_id,
            change_time:time,
            status:(RbacRoleUserStatus::Delete as i8),
        });
        let db = &self.db;
        let res = db_option_executor!(
            db,
            {
                Update::< RbacRoleUserModel, _>::new(ddata)
                    .execute_by_where(
                        &lsys_core::db::WhereOption::Where(sql_format!(
                            "role_id ={} and user_id  in ({})",
                            role.id,
                            user_id_vec
                        )),
                        db.as_executor(),
                    )
                    .await?
            },
            transaction,
            db
        );

        self.cache()
            .clear_access(role, Some(&[]), Some(user_id_vec))
            .await;

        self.logger
            .add(
                &LogRoleUser {
                    action: "del",
                    name: role.role_name.as_str(),
                    add_user: None,
                    user_id: role.user_id,
                    del_user: Some(user_id_vec.to_owned()),
                },
                Some(role.id),
                Some(del_user_id),
                None,
                env_data,
            )
            .await;
        Ok(res.rows_affected())
    }
}
