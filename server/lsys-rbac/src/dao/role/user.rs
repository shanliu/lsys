//RBAC中角色相关实现

use lsys_core::utils::{RequestEnv, now_time};

use serde::Serialize;

use lsys_core::db::{BatchInsert, Insert, QueryBuilderExt, TableMeta, Update};
use sqlx::{MySql, QueryBuilder};

use lsys_core::db::OptionTxExecutor;
use sqlx::Transaction;

use super::{RbacRole, logger::LogRoleUser};
use crate::dao::role::fluent_message;
use crate::{
    dao::result::{RbacError, RbacResult},
    model::{RbacRoleModel, RbacRoleUserModel, RbacRoleUserRange, RbacRoleUserStatus},
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

        let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
            "select id,user_id from {}",
            RbacRoleUserModel::table_name(),
        ));
        qb.push_where().field_in_copied("user_id", &user_id_vec);
        qb.push_and().field_eq("role_id", role.id);
        let user_res = qb
            .build_query_as::<(u64, u64)>()
            .fetch_all(&self.db)
            .await?;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let nowtime = now_time().unwrap_or_default();
        let mut batch = BatchInsert::<_, RbacRoleUserModel>::new();
        for RoleAddUser { user_id, timeout } in user_vec.iter() {
            let mut is_updata = false;
            for (itemid, uid) in user_res.iter() {
                if uid == user_id {
                    if let Err(err) = Update::<_, RbacRoleUserModel>::new()
                        .set(RbacRoleUserModel::ROLE_ID, role.id)
                        .set(RbacRoleUserModel::CHANGE_TIME, nowtime)
                        .set(RbacRoleUserModel::CHANGE_USER_ID, add_user_id)
                        .set(RbacRoleUserModel::STATUS, RbacRoleUserStatus::Enable as i8)
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
                    Insert::<_, RbacRoleUserModel>::new()
                        .set(RbacRoleUserModel::USER_ID, *user_id)
                        .set(RbacRoleUserModel::TIMEOUT, *timeout)
                        .set(RbacRoleUserModel::ROLE_ID, role.id)
                        .set(RbacRoleUserModel::CHANGE_TIME, nowtime)
                        .set(RbacRoleUserModel::CHANGE_USER_ID, add_user_id)
                        .set(RbacRoleUserModel::STATUS, RbacRoleUserStatus::Enable as i8),
                );
            }
        }
        if !batch.is_empty()
            && let Err(err) = batch.execute(&mut *db).await
        {
            db.rollback().await?;
            return Err(err.into());
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
        let res = Update::<_, RbacRoleUserModel>::new()
            .set(RbacRoleUserModel::CHANGE_USER_ID, del_user_id)
            .set(RbacRoleUserModel::CHANGE_TIME, time)
            .set(RbacRoleUserModel::STATUS, RbacRoleUserStatus::Delete as i8)
            .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                qb.push_where()
                    .field_eq("role_id", role.id)
                    .push_and()
                    .field_in_copied("user_id", user_id_vec);
            })
            .await?;

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
