use lsys_core::cache::LocalCache;
use lsys_core::{
    get_message, impl_cache_fetch_vec, impl_dao_fetch_one_by_one, now_time, FluentMessage,
    PageParam,
};

use sqlx::{Acquire, FromRow, MySql, Pool, Row, Transaction};
use sqlx_model::{
    executor_option, model_option_set, sql_format, Insert, ModelTableName, Select, SqlExpr,
    SqlQuote, Update,
};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use crate::model::{
    RbacResModel, RbacResModelRef, RbacResOpModel, RbacResOpModelRef, RbacResOpStatus,
    RbacResStatus, RbacTagsModel, RbacTagsSource,
};

use super::{RbacRole, RbacTags, UserRbacError, UserRbacResult};

pub struct RbacRes {
    fluent: Arc<FluentMessage>,
    db: Pool<MySql>,
    tags: Arc<RbacTags>,
    cache_key_res: Arc<LocalCache<ResKey, Option<RbacResData>>>, // res_key:res edit,res_op all
    role: Arc<RbacRole>,
}

#[derive(Clone, Debug)]
pub struct RbacResData {
    pub res: RbacResModel,
    pub ops: Vec<RbacResOpModel>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResKey {
    pub res_key: String, //资源KEY
    pub user_id: u64,    //资源用户ID
}

#[derive(Clone, Debug)]
pub struct ResOp {
    pub name: String, //操作名
    pub key: String,  //操作key
}

impl ToString for ResKey {
    fn to_string(&self) -> String {
        format!("{}-{}", self.user_id, self.res_key)
    }
}

impl FromStr for ResKey {
    type Err = UserRbacError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut token_split = s.split('-');
        let user_id = token_split
            .next()
            .ok_or_else(|| UserRbacError::System(format!("res key parse fail:{}", s)))?;
        let user_id = user_id
            .parse::<u64>()
            .map_err(|e| UserRbacError::System(e.to_string()))?;
        let res_key = token_split
            .next()
            .ok_or_else(|| UserRbacError::System("token is not split fail:token".to_string()))?
            .to_string();
        Ok(ResKey { user_id, res_key })
    }
}

impl RbacRes {
    pub fn new(
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        tags: Arc<RbacTags>,
        role: Arc<RbacRole>,
        cache_key_res: Arc<LocalCache<ResKey, Option<RbacResData>>>,
    ) -> Self {
        Self {
            cache_key_res,
            db,
            tags,
            fluent,
            role,
        }
    }
    impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        RbacResModel,
        UserRbacResult<RbacResModel>,
        id,
        "id={id} and status = {status}",
        status = RbacResStatus::Enable
    );
    /// 获取指定用户和ID的数量
    pub async fn find_by_op_ids(
        &self,
        op_id: &[u64],
    ) -> UserRbacResult<Vec<(RbacResModel, RbacResOpModel)>> {
        if op_id.is_empty() {
            return Ok(vec![]);
        }

        let sql = sql_format!(
            "select res.*,
            res_op.id as op_id,
            res_op.name as op_name,
            res_op.op_key as op_op_key,
            res_op.status as op_status,
            res_op.change_user_id as op_change_user_id,
            res_op.change_time as op_change_time
            from {} as res 
            join {} as res_op
            on res.id=res_op.res_id
            where res.status ={} and res_op.status={} and res_op.id in ({})
            order by id desc",
            RbacResModel::table_name(),
            RbacResOpModel::table_name(),
            RbacResStatus::Enable as i8,
            RbacResOpStatus::Enable as i8,
            op_id
        );
        let res = sqlx::query(sql.as_str())
            .try_map(|row: sqlx::mysql::MySqlRow| {
                let op_id = row.try_get::<u64, &str>("op_id").unwrap_or(0);
                let op_name = row
                    .try_get::<String, &str>("op_name")
                    .unwrap_or_else(|_| "".to_string());
                let op_key = row
                    .try_get::<String, &str>("op_op_key")
                    .unwrap_or_else(|_| "".to_string());
                let op_status = row.try_get::<i8, &str>("op_status").unwrap_or(0);
                let op_change_user_id = row.try_get::<u64, &str>("op_change_user_id").unwrap_or(0);
                let op_change_time = row.try_get::<u64, &str>("op_change_time").unwrap_or(0);
                let res = RbacResModel::from_row(&row)?;
                let res_id = res.id;
                Ok((
                    res,
                    RbacResOpModel {
                        id: op_id,
                        name: op_name,
                        op_key,
                        res_id,
                        status: op_status,
                        change_user_id: op_change_user_id,
                        change_time: op_change_time,
                    },
                ))
            })
            .fetch_all(&self.db)
            .await?;
        Ok(res)
    }
    /// 获取指定用户和ID的数量
    pub async fn get_count(&self, user_id: u64,res_name:&Option<String>, res_ids: &Option<Vec<u64>>) -> UserRbacResult<i64> {
        let mut sql = sql_format!(
            "select count(*) from {} where user_id = ? and status=?",
            RbacResModel::table_name()
        );
        if let Some(ref name) = res_name {
            sql += sql_format!(" and name like {}", format!("%{}%",name)).as_str();
        }
        if let Some(ref rid) = res_ids {
            if rid.is_empty() {
                return Ok(0);
            } else {
                sql += sql_format!(" and id in ({})", rid).as_str();
            }
        }
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        query = query.bind(user_id).bind(RbacResStatus::Enable as i8);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    /// 获取指定用户跟ID的资源
    pub async fn get_res(
        &self,
        user_id: u64,
        res_name:&Option<String>,
        res_ids: &Option<Vec<u64>>,
        page: &Option<PageParam>,
    ) -> UserRbacResult<Vec<RbacResModel>> {
        let mut sql = "user_id = ? and status=?".to_string();
        if let Some(ref rid) = res_ids {
            if rid.is_empty() {
                return Ok(vec![]);
            } else {
                sql += &sql_format!(" and id in ({})", rid);
            }
        }
        if let Some(name) = res_name {
            sql += sql_format!(" and name like {}", format!("%{}%",name)).as_str();
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }
        let data = Select::type_new::<RbacResModel>()
            .fetch_all_by_where_call::<RbacResModel, _, _>(
                sql,
                |mut tmp, _| {
                    tmp = tmp.bind(user_id).bind(RbacResStatus::Enable as i8);
                    tmp
                },
                &self.db,
            )
            .await?;
        Ok(data)
    }
    /// 指定用户的所有TAG
    pub async fn user_res_tags(&self, user_id: u64) -> UserRbacResult<Vec<(String, i64)>> {
        self.tags
            .group_by_user_id(user_id, RbacTagsSource::Res)
            .await
    }
    /// 获取资源的TAG
    pub async fn res_get_tags(
        &self,
        res_ids: &[u64],
    ) -> UserRbacResult<BTreeMap<u64, Vec<RbacTagsModel>>> {
        let data = self.tags.find_by_ids(res_ids, RbacTagsSource::Res).await?;
        let mut result = BTreeMap::<u64, Vec<RbacTagsModel>>::new();
        for res_op in data {
            result.entry(res_op.from_id).or_default().push(res_op);
        }
        Ok(result)
    }
    /// 设置资源的TAG
    pub async fn res_set_tags<'t>(
        &self,
        res: &RbacResModel,
        tags: &[String],
        user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserRbacResult<()> {
        let tags = {
            let mut tout = Vec::with_capacity(tags.len());
            for tmp in tags.iter() {
                tout.push(check_length!(&self.fluent, tmp, "name", 32));
            }
            tout
        };
        self.tags
            .set_tags(
                res.id,
                res.user_id,
                RbacTagsSource::Res,
                &tags,
                user_id,
                transaction,
            )
            .await
    }

    /// 添加资源
    pub async fn add_res<'t>(
        &self,
        user_id: u64,
        name: String,
        key: String,
        add_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserRbacResult<u64> {
        let key = check_length!(&self.fluent, key, "key", 32);
        let name = check_length!(&self.fluent, name, "name", 32);

        let res = Select::type_new::<RbacResModel>()
            .fetch_one_by_where_call::<RbacResModel, _, _>(
                "user_id=? and res_key=? and status=?".to_string(),
                |mut res, _| {
                    res = res.bind(user_id);
                    res = res.bind(key.clone());
                    res = res.bind(RbacResStatus::Enable as i8);
                    res
                },
                &self.db,
            )
            .await;

        match res {
            Ok(rm) => Err(UserRbacError::System(
                get_message!(&self.fluent,"rbac-res-exits","res [{$key}] already exists,name is:{$name}",[
                    "key"=>key.clone(),
                    "name"=>rm.name
                ]),
            )),
            Err(sqlx::Error::RowNotFound) => {
                let time = now_time().unwrap_or_default();
                let idata = model_option_set!(RbacResModelRef,{
                    name:name,
                    res_key:key,
                    user_id:user_id,
                    change_time:time,
                    add_user_id:add_user_id,
                    change_user_id:0,
                    status:(RbacResStatus::Enable as i8),
                });
                let id = executor_option!(
                    {
                        let res = Insert::<sqlx::MySql, RbacResModel, _>::new(idata)
                            .execute(db)
                            .await?;
                        res.last_insert_id()
                    },
                    transaction,
                    &self.db,
                    db
                );
                self.cache_key_res
                    .clear(&ResKey {
                        res_key: key,
                        user_id,
                    })
                    .await;
                Ok(id)
            }
            Err(e) => Err(e)?,
        }
    }
    /// 编辑资源
    pub async fn edit_res<'t>(
        &self,
        res: &RbacResModel,
        name: Option<String>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserRbacResult<u64> {
        let time = now_time().unwrap_or_default();
        let mut change = sqlx_model::model_option_set!(RbacResModelRef,{
            change_user_id:change_user_id,
            change_time:time,
        });
        let opt_name = if let Some(tname) = name {
            Some(check_length!(&self.fluent, tname, "name", 32))
        } else {
            None
        };
        change.name = opt_name.as_ref();
        let db = &self.db;
        let fout = executor_option!(
            {
                let out = Update::<sqlx::MySql, RbacResModel, _>::new(change)
                    .execute_by_pk(res, db)
                    .await?;
                Ok(out.rows_affected())
            },
            transaction,
            db,
            db
        );
        self.cache_key_res
            .clear(&ResKey {
                res_key: res.res_key.clone(),
                user_id: res.user_id,
            })
            .await;
        fout
    }
    /// 删除资源
    pub async fn del_res<'t>(
        &self,
        res: &RbacResModel,
        delete_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserRbacResult<()> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(RbacResModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacResStatus::Delete as i8)
        });
        let ops = self
            .res_get_ops(&[res.id])
            .await?
            .get(&res.id)
            .map(|e| e.to_owned())
            .unwrap_or_else(Vec::new);

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, RbacResModel, _>::new(change)
            .execute_by_pk(res, &mut db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        let tmp = self
            .tags
            .del_tags(res.id, RbacTagsSource::Res, delete_user_id, Some(&mut db))
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        let change = sqlx_model::model_option_set!(RbacResOpModelRef,{
            change_user_id:delete_user_id,
            change_time:time,
            status:(RbacResOpStatus::Delete as i8)
        });
        let tmp = Update::<sqlx::MySql, RbacResOpModel, _>::new(change)
            .execute_by_where_call("res_id=?", |temp, _| temp.bind(res.id), &mut db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        let tmp = self
            .role
            .all_role_del_ops(
                &ops.into_iter().map(|e| e.id).collect::<Vec<u64>>(),
                delete_user_id,
                Some(&mut db),
            )
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }
        db.commit().await?;
        self.cache_key_res
            .clear(&ResKey {
                res_key: res.res_key.clone(),
                user_id: res.user_id,
            })
            .await;
        Ok(())
    }
    /// 获取资源操作
    pub async fn res_get_ops(
        &self,
        res_ids: &[u64],
    ) -> UserRbacResult<BTreeMap<u64, Vec<RbacResOpModel>>> {
        if res_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let db = &self.db;
        let data = Select::type_new::<RbacResOpModel>()
            .fetch_all_by_where_call::<RbacResOpModel, _, _>(
                sql_format!("res_id IN ({}) and status=?", res_ids).to_string(),
                |tmp, _| tmp.bind(RbacResStatus::Enable as i8),
                db,
            )
            .await?;
        let mut result = BTreeMap::<u64, Vec<RbacResOpModel>>::new();
        for res_op in data {
            result.entry(res_op.res_id).or_default().push(res_op);
        }
        Ok(result)
    }
    /// 设置资源操作
    pub async fn res_set_ops<'t>(
        &self,
        res: &RbacResModel,
        ops: Vec<ResOp>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserRbacResult<()> {
        let time = now_time().unwrap_or_default();
        let db = &self.db;

        let ops = {
            let mut tout = Vec::with_capacity(ops.len());
            for mut tmp in ops.into_iter() {
                let key = tmp.key;
                let name = tmp.name;
                tmp.key = check_length!(&self.fluent, key, "key", 32);
                tmp.name = check_length!(&self.fluent, name, "name", 32);
                tout.push(tmp);
            }
            tout
        };

        let fops = Select::type_new::<RbacResOpModel>()
            .fetch_all_by_where_call::<RbacResOpModel, _, _>(
                "res_id=? and status=?".to_string(),
                |tmp, _| tmp.bind(res.id).bind(RbacResOpStatus::Enable as i8),
                db,
            )
            .await?;

        let mut del_op = vec![];
        let mut change_op = vec![];
        for iop in fops.iter() {
            let mut find = false;
            for resop in ops.iter() {
                if *resop.key == iop.op_key {
                    find = true;
                    if *resop.name != iop.name {
                        change_op.push((iop.id, resop.name.to_owned()));
                    }
                    break;
                }
            }
            if !find {
                del_op.push(iop.id);
            }
        }

        let sop = fops.into_iter().map(|e| e.op_key).collect::<Vec<_>>();
        let add_op = ops
            .into_iter()
            .filter(|e| !sop.contains(&e.key))
            .collect::<Vec<_>>();

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        if !add_op.is_empty() {
            let mut idata = Vec::with_capacity(add_op.len());
            for resop in add_op.iter() {
                let mut item = model_option_set!(RbacResOpModelRef,{
                    res_id:res.id,
                    change_time:time,
                    change_user_id:change_user_id,
                    status:(RbacResOpStatus::Enable as i8),
                });
                item.name = Some(&resop.name);
                item.op_key = Some(&resop.key);
                idata.push(item);
            }
            let tmp = Insert::<sqlx::MySql, RbacResOpModel, _>::new_vec(idata)
                .execute(&mut db)
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            };
        }
        if !del_op.is_empty() {
            let ddata = model_option_set!(RbacResOpModelRef,{
                change_time:time,
                change_user_id:change_user_id,
                status:(RbacResOpStatus::Delete as i8),
            });

            let tmp = Update::<sqlx::MySql, RbacResOpModel, _>::new(ddata)
                .execute_by_where(Some(sql_format!("id in ({})", del_op)), &mut db)
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            };
        }

        for (cid, cn) in change_op {
            let ddata = model_option_set!(RbacResOpModelRef,{
                name:cn,
            });
            let tmp = Update::<sqlx::MySql, RbacResOpModel, _>::new(ddata)
                .execute_by_where_call("id = ?", |temp, _| temp.bind(cid), &mut db)
                .await;
            if let Err(e) = tmp {
                db.rollback().await?;
                return Err(e)?;
            }
        }

        let tmp = self
            .role
            .all_role_del_ops(&del_op, change_user_id, Some(&mut db))
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e)?;
        }

        db.commit().await?;
        self.cache_key_res
            .clear(&ResKey {
                res_key: res.res_key.clone(),
                user_id: res.user_id,
            })
            .await;
        Ok(())
    }
    /// 根据资源KEY获取资源
    pub async fn find_by_keys(
        &self,
        keys: &[ResKey],
    ) -> UserRbacResult<HashMap<ResKey, Option<RbacResData>>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }
        let mut where_sql = Vec::with_capacity(keys.len());
        for rkey in keys {
            where_sql.push(sql_format!(
                "(res_key ={} and user_id={})",
                rkey.res_key,
                rkey.user_id,
            ));
        }
        let sql = sql_format!(
            "select * from {} where
            ({}) and status ={} order by id desc",
            RbacResModel::table_name(),
            SqlExpr(where_sql.join(" or ")),
            RbacResStatus::Enable as i8
        );
        let res = sqlx::query_as::<_, RbacResModel>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        let res_op = if !res.is_empty() {
            let res_id = res.iter().map(|res| res.id).collect::<Vec<_>>();
            Select::type_new::<RbacResOpModel>()
                .fetch_all_by_where_call::<RbacResOpModel, _, _>(
                    sql_format!("res_id in ({}) and status =? order by id desc", res_id),
                    |mut res, _| {
                        res = res.bind(RbacResOpStatus::Enable as i8);
                        res
                    },
                    &self.db,
                )
                .await?
        } else {
            vec![]
        };
        let mut out = HashMap::with_capacity(res.len());
        for res_ in res {
            let mut op_vec = vec![];
            for op_ in res_op.iter() {
                if op_.res_id != res_.id {
                    continue;
                }
                op_vec.push(op_.to_owned());
            }
            out.entry(ResKey {
                user_id: res_.user_id,
                res_key: res_.res_key.clone(),
            })
            .or_insert(Some(RbacResData {
                res: res_,
                ops: op_vec,
            }));
        }
        for rkey in keys {
            out.entry(rkey.to_owned()).or_insert(None);
        }
        Ok(out)
    }
    pub fn cache(&self) -> RbacResCache<'_> {
        RbacResCache { res: self }
    }
}

pub struct RbacResCache<'t> {
    pub res: &'t RbacRes,
}

impl<'t> RbacResCache<'t> {
    impl_cache_fetch_vec!(
        find_by_keys,
        res,
        cache_key_res,
        ResKey,
        UserRbacResult<HashMap<ResKey, Option<RbacResData>>>
    );
}
