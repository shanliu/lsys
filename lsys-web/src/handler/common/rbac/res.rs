use std::sync::Arc;

use crate::{
    PageParam, {JsonData, JsonResult},
};

use lsys_rbac::{
    dao::{RbacDao, RbacRes, ResKey, ResOp, ResParam},
    model::{RbacResModel, RbacResOpModel, RbacTagsModel},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Transaction;
use sqlx_model::Select;

#[derive(Debug, Deserialize)]
pub struct ResOpParam {
    name: String,
    key: String,
}

impl From<ResOpParam> for ResOp {
    fn from(p: ResOpParam) -> Self {
        ResOp {
            name: p.name,
            key: p.key,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResAddParam {
    pub user_id: Option<u64>,
    pub name: String,
    pub key: String,
    pub ops: Option<Vec<ResOpParam>>,
    pub tags: Option<Vec<String>>,
}
pub async fn rbac_res_add(
    param: ResAddParam,
    rbac_dao: &RbacDao,
    user_id: u64,
) -> JsonResult<JsonData> {
    let add_user_id = param.user_id.unwrap_or(user_id);

    rbac_dao
        .rbac
        .access
        .check(user_id, &[], &res_data!(UserResEdit(add_user_id)))
        .await?;

    let dao = &rbac_dao.rbac.res;
    let mut transaction = rbac_dao.db.begin().await?;
    let id = match dao
        .add_res(
            add_user_id,
            param.name,
            param.key,
            user_id,
            Some(&mut transaction),
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            transaction.rollback().await?;
            return Err(e.into());
        }
    };

    let res = Select::type_new::<RbacResModel>()
        .fetch_one_by_where_call::<RbacResModel, _, _>(
            "id=?".to_string(),
            |mut res, _| {
                res = res.bind(id.to_owned());
                res
            },
            &mut transaction,
        )
        .await?;
    if let Err(e) = set_attr(dao, &res, param.ops, param.tags, user_id, &mut transaction).await {
        transaction.rollback().await?;
        return Err(e);
    };
    transaction.commit().await?;
    Ok(JsonData::message("set succ").set_data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct ResEditParam {
    pub res_id: u64,
    pub key: Option<String>,
    pub name: Option<String>,
    pub ops: Option<Vec<ResOpParam>>,
    pub tags: Option<Vec<String>>,
}

pub async fn rbac_res_edit(
    param: ResEditParam,
    rbac_dao: &RbacDao,
    user_id: u64,
) -> JsonResult<JsonData> {
    let dao = &rbac_dao.rbac.res;
    let res = dao.find_by_id(&param.res_id).await?;
    rbac_dao
        .rbac
        .access
        .check(user_id, &[], &res_data!(UserResEdit(res.user_id)))
        .await?;
    let mut transaction = rbac_dao.db.begin().await?;
    if let Err(e) = dao
        .edit_res(&res, param.name, user_id, Some(&mut transaction))
        .await
    {
        transaction.rollback().await?;
        return Err(e.into());
    };
    if let Err(e) = set_attr(dao, &res, param.ops, param.tags, user_id, &mut transaction).await {
        transaction.rollback().await?;
        return Err(e);
    };
    transaction.commit().await?;

    Ok(JsonData::message("save succ"))
}

async fn set_attr<'t>(
    dao: &Arc<RbacRes>,
    res: &RbacResModel,
    ops: Option<Vec<ResOpParam>>,
    tags: Option<Vec<String>>,
    change_user_id: u64,
    transaction: &mut Transaction<'t, sqlx::MySql>,
) -> JsonResult<()> {
    if let Some(tmp) = ops {
        let ops = tmp
            .into_iter()
            .map(|e| ResOp {
                name: e.name,
                key: e.key,
            })
            .collect();
        dao.res_set_ops(res, ops, change_user_id, Some(transaction))
            .await?;
    }
    if let Some(ref tmp) = tags {
        dao.res_set_tags(res, tmp, change_user_id, Some(transaction))
            .await?
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ResDeleteParam {
    pub res_id: u64,
}
pub async fn rbac_res_delete(
    param: ResDeleteParam,
    rbac_dao: &RbacDao,
    user_id: u64,
) -> JsonResult<JsonData> {
    let resdao = &rbac_dao.rbac.res;
    let res = resdao.find_by_id(&param.res_id).await?;
    rbac_dao
        .rbac
        .access
        .check(user_id, &[], &res_data!(UserResEdit(res.user_id)))
        .await?;
    resdao.del_res(&res, user_id, None).await?;
    Ok(JsonData::message("del succ"))
}

#[derive(Debug, Deserialize)]
pub struct ResTagsParam {
    pub user_id: Option<u64>,
}
pub async fn rbac_res_tags(
    param: ResTagsParam,
    rbac_dao: &RbacDao,
    user_id: u64,
) -> JsonResult<JsonData> {
    let see_user_id = param.user_id.unwrap_or(user_id);
    rbac_dao
        .rbac
        .access
        .check(user_id, &[], &res_data!(UserResView(see_user_id)))
        .await?;
    let out = rbac_dao.rbac.res.user_res_tags(see_user_id).await?;
    Ok(JsonData::message("tags data").set_data(json!({ "data": out })))
}

#[derive(Debug, Deserialize)]
pub struct ResListDataParam {
    pub count_num: Option<bool>,
    pub user_id: Option<u64>,
    pub res_name: Option<String>,
    pub res_id: Option<Vec<u64>>,
    pub tags_filter: Option<Vec<String>>,
    pub tags: bool,
    pub ops: bool,
    pub page: Option<PageParam>,
}

#[derive(Debug, Serialize)]
pub struct ResData {
    res: RbacResModel,
    tags: Option<Vec<RbacTagsModel>>,
    ops: Option<Vec<RbacResOpModel>>,
}
pub async fn rbac_res_list_data(
    param: ResListDataParam,
    rbac_dao: &RbacDao,
    user_id: u64,
) -> JsonResult<JsonData> {
    let see_user_id = param.user_id.unwrap_or(user_id);
    rbac_dao
        .rbac
        .access
        .check(user_id, &[], &res_data!(UserResView(see_user_id)))
        .await?;

    let dao = &rbac_dao.rbac.data;
    let res = dao
        .res_data(&ResParam {
            user_id: see_user_id,
            res_id: &param.res_id,
            res_name: &param.res_name,
            filter_tags: &param.tags_filter,
            out_ops: param.ops,
            out_tags: param.tags,
            page: &param.page.map(|e| e.into()),
        })
        .await?;
    let out = res
        .into_iter()
        .map(|e| ResData {
            res: e.0,
            tags: if param.tags { Some(e.2) } else { None },
            ops: if param.ops { Some(e.1) } else { None },
        })
        .collect::<Vec<ResData>>();
    let count = if param.count_num.unwrap_or(false) {
        Some(
            dao.res_count(
                see_user_id,
                &param.res_name,
                &param.res_id,
                &param.tags_filter,
            )
            .await?,
        )
    } else {
        None
    };
    Ok(JsonData::message("res data").set_data(json!({ "data": out,"count":count})))
}

#[derive(Debug, Serialize)]
pub struct ShowResItem {
    pub op: String,
    pub must_authorize: bool, //是否必须被验证权限，为否且不被管理时不验证权限
    pub is_must: bool,        //是否已经被管理的权限，即数据库里有权限的记录
}
#[derive(Debug, Serialize)]
pub struct ShowResData {
    pub res: String,
    pub ops: Vec<ShowResItem>,
    pub user_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ResAllParam {
    pub user_id: Option<u64>,
}

//资源授权
pub async fn rbac_all_res_list(
    param: ResAllParam,
    rbac_dao: &RbacDao,
    user_id: u64,
) -> JsonResult<JsonData> {
    let see_user_id = param.user_id.unwrap_or(user_id);
    rbac_dao
        .rbac
        .access
        .check(user_id, &[], &res_data!(UserResAllView(see_user_id)))
        .await?;
    let data = crate::dao::access::ResData::all_res(see_user_id);
    let keys = data
        .iter()
        .flat_map(|e| {
            e.ops
                .iter()
                .map(|t| ResKey {
                    res_key: t.op.to_owned(),
                    user_id: e.user_id,
                })
                .collect::<Vec<ResKey>>()
        })
        .collect::<Vec<ResKey>>();
    let dbres = rbac_dao.rbac.res.find_by_keys(&keys).await?;
    let out = data
        .iter()
        .map(|e| {
            let ops = e
                .ops
                .iter()
                .map(|t| ShowResItem {
                    op: t.op.to_owned(),
                    must_authorize: t.must_authorize,
                    is_must: dbres
                        .iter()
                        .filter_map(|e| e.1.as_ref().map(|t| (e.0, t)))
                        .any(|o| {
                            let t1 = ResKey {
                                res_key: t.op.clone(),
                                user_id: e.user_id,
                            };
                            if *o.0 == t1 {
                                o.1.ops.iter().any(|n| n.op_key == t.op)
                            } else {
                                false
                            }
                        }),
                })
                .collect::<Vec<ShowResItem>>();
            ShowResData {
                user_id: if e.user_id > 0 { Some(e.user_id) } else { None },
                res: e.res.to_owned(),
                ops,
            }
        })
        .collect::<Vec<ShowResData>>();
    Ok(JsonData::message("res data").set_data(json!({ "data": out })))
}
