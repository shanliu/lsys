//RBAC中资源相关实现
use lsys_core::fluent_message;

use crate::dao::result::{RbacError, RbacResult};
use crate::model::{
    RbacOpModel, RbacOpResModel, RbacOpResModelRef, RbacOpResStatus, RbacResModel, RbacResStatus,
};

use sqlx::{FromRow, Row};
use std::vec;

use lsys_core::{now_time, RequestEnv};

use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update, WhereOption};
use lsys_core::{model_option_set, sql_format};
use sqlx::{Acquire, Transaction};

use super::logger::LogResOp;
use super::RbacRes;

//资源的跟对应可用操作相关实现

//资源管理
impl RbacRes {
    //获取某资源可用操作
    pub async fn add_op(
        &self,
        res: &RbacResModel,
        op_vec: &[RbacOpModel],
        add_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        if op_vec.is_empty() {
            return Ok(());
        }
        for op in op_vec {
            if op.user_id != res.user_id {
                return Err(RbacError::System(fluent_message!("rbac-res-add-bad-op",{
                    "res_name":&res.res_name,
                    "op_name":&op.op_name,
                    "user_id":op.user_id
                })));
            }
        }

        let op_res = sqlx::query_as::<_, (u64, u64)>(&sql_format!(
            "select id,op_id from {} where res_type={} and user_id={} and op_id in ({})",
            RbacOpResModel::table_name(),
            &res.res_type,
            res.user_id,
            op_vec.iter().map(|e| e.id).collect::<Vec<_>>()
        ))
        .fetch_all(&self.db)
        .await?;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let nowtime = now_time().unwrap_or_default();
        let mut add_item = vec![];
        for op in op_vec {
            let mut is_updata = false;
            for (itemid, op_id) in op_res.iter() {
                if *op_id == op.id {
                    let item = model_option_set!(RbacOpResModelRef,{
                        change_time:nowtime,
                        change_user_id:add_user_id,
                        status:(RbacOpResStatus::Enable as i8),
                    });
                    if let Err(err) = Update::<sqlx::MySql, RbacOpResModel, _>::new(item)
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
                add_item.push(model_option_set!(RbacOpResModelRef,{
                    op_id:op.id,
                    res_type:res.res_type,
                    user_id:res.user_id,
                    change_time:nowtime,
                    change_user_id:add_user_id,
                    status:(RbacOpResStatus::Enable as i8),
                }));
            }
        }
        if !add_item.is_empty() {
            if let Err(err) = Insert::<sqlx::MySql, RbacOpResModel, _>::new_vec(add_item)
                .execute(&mut *db)
                .await
            {
                db.rollback().await?;
                return Err(err.into());
            }
        }
        db.commit().await?;

        self.logger
            .add(
                &LogResOp {
                    action: "add",
                    res_type: res.res_type.as_str(),
                    user_id: res.user_id,
                    res_data: op_vec.iter().map(|e| e.id).collect::<Vec<_>>(),
                },
                Some(res.id),
                Some(add_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    //获取某资源可用操作
    pub async fn del_op(
        &self,
        res: &RbacResModel,
        op_id_vec: &[u64],
        del_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<u64> {
        if op_id_vec.is_empty() {
            return Ok(0);
        }

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let tmp = self
            .role
            .role_remove_perm(res, op_id_vec, del_user_id, Some(&mut db), env_data)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        let time = now_time().unwrap_or_default();
        let ddata = model_option_set!(RbacOpResModelRef,{
            change_user_id:del_user_id,
            change_time:time,
            status:(RbacOpResStatus::Delete as i8),
        });
        let change = Update::<sqlx::MySql, RbacOpResModel, _>::new(ddata)
            .execute_by_where(
                &lsys_core::db::WhereOption::Where(sql_format!(
                    "res_type={} and user_id={} and op_id in ({})",
                    res.res_type,
                    res.user_id,
                    op_id_vec
                )),
                &mut *db,
            )
            .await?;

        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        db.commit().await?;

        self.logger
            .add(
                &LogResOp {
                    action: "del",
                    user_id: res.user_id,
                    res_type: res.res_type.as_str(),
                    res_data: op_id_vec.to_vec(),
                },
                Some(res.id),
                Some(del_user_id),
                None,
                env_data,
            )
            .await;
        Ok(change.rows_affected())
    }

    pub(crate) async fn res_remove_op(
        &self,
        op: &RbacOpModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> RbacResult<()> {
        let mut res_id = 0;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        loop {
            let res_data = sqlx::query(&sql_format!(
                "select res.*,op_res.op_id from {} as res 
                    join {} as op_res on res.res_type=op_res.res_type and res.user_id=op_res.user_id
                    where op_res.op_id={} and res.status={} and op_res.status={}",
                RbacResModel::table_name(),
                RbacOpResModel::table_name(),
                op.id,
                RbacResStatus::Enable,
                RbacOpResStatus::Enable,
            ))
            .try_map(
                |row: sqlx::mysql::MySqlRow| match RbacResModel::from_row(&row) {
                    Ok(res) => {
                        let op_id = row.try_get::<u64, &str>("op_id").unwrap_or_default();
                        res_id = res.id;
                        Ok((op_id, res))
                    }
                    Err(err) => Err(err),
                },
            )
            .fetch_all(&self.db)
            .await?;
            if res_data.is_empty() {
                break;
            }
            let mut group_data: Vec<(RbacResModel, Vec<u64>)> = vec![];
            for tmp in res_data {
                if let Some(itmp) = group_data.iter_mut().find(|e| e.0.id == tmp.1.id) {
                    itmp.1.push(tmp.0);
                } else {
                    group_data.push((tmp.1, vec![tmp.0]));
                }
            }
            for (res, op_id_vec) in group_data {
                self.del_op(&res, &op_id_vec, delete_user_id, Some(&mut db), env_data)
                    .await?;
            }
        }
        db.commit().await?;
        Ok(())
    }
}
