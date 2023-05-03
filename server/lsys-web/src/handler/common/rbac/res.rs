use std::sync::Arc;

use crate::{
    handler::access::{AccessResEdit, AccessResView},
    PageParam, {JsonData, JsonResult},
};

use lsys_core::RequestEnv;
use lsys_rbac::{
    dao::{RbacDao, RbacRes, ResOp, ResParam, ResTpl},
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
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    let add_user_id = param.user_id.unwrap_or(user_id);
    rbac_dao
        .rbac
        .check(
            &AccessResEdit {
                user_id,
                res_user_id: add_user_id,
            },
            None,
        )
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
            env_data,
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
            "id=?",
            |mut res, _| {
                res = res.bind(id.to_owned());
                res
            },
            &mut transaction,
        )
        .await?;
    if let Err(e) = set_attr(
        dao,
        &res,
        param.ops,
        param.tags,
        user_id,
        &mut transaction,
        env_data,
    )
    .await
    {
        transaction.rollback().await?;
        return Err(e);
    };
    transaction.commit().await?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct ResEditParam {
    pub res_id: u64,
    pub name: Option<String>,
    pub ops: Option<Vec<ResOpParam>>,
    pub tags: Option<Vec<String>>,
}

pub async fn rbac_res_edit(
    param: ResEditParam,
    rbac_dao: &RbacDao,
    user_id: u64,
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    let dao = &rbac_dao.rbac.res;
    let res = dao.find_by_id(&param.res_id).await?;
    rbac_dao
        .rbac
        .check(
            &AccessResEdit {
                user_id,
                res_user_id: res.user_id,
            },
            None,
        )
        .await?;

    let mut transaction = rbac_dao.db.begin().await?;
    if let Err(e) = dao
        .edit_res(&res, param.name, user_id, Some(&mut transaction), env_data)
        .await
    {
        transaction.rollback().await?;
        return Err(e.into());
    };
    if let Err(e) = set_attr(
        dao,
        &res,
        param.ops,
        param.tags,
        user_id,
        &mut transaction,
        env_data,
    )
    .await
    {
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
    env_data: Option<&RequestEnv>,
) -> JsonResult<()> {
    if let Some(tmp) = ops {
        let ops = tmp
            .into_iter()
            .map(|e| ResOp {
                name: e.name,
                key: e.key,
            })
            .collect();
        dao.res_set_ops(res, ops, change_user_id, Some(transaction), env_data)
            .await?;
    }
    if let Some(ref tmp) = tags {
        dao.res_set_tags(res, tmp, change_user_id, Some(transaction), env_data)
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
    env_data: Option<&RequestEnv>,
) -> JsonResult<JsonData> {
    let resdao = &rbac_dao.rbac.res;
    let res = resdao.find_by_id(&param.res_id).await?;
    rbac_dao
        .rbac
        .check(
            &AccessResEdit {
                user_id,
                res_user_id: res.user_id,
            },
            None,
        )
        .await?;
    resdao.del_res(&res, user_id, None, env_data).await?;
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
        .check(
            &AccessResView {
                user_id,
                res_user_id: see_user_id,
            },
            None,
        )
        .await?;
    let out = rbac_dao.rbac.res.user_res_tags(see_user_id).await?;
    Ok(JsonData::data(json!({ "data": out })))
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
pub struct ResShowData {
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
        .check(
            &AccessResView {
                user_id,
                res_user_id: see_user_id,
            },
            None,
        )
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
            page: &Some(param.page.unwrap_or_default().into()),
        })
        .await?;
    let out = res
        .into_iter()
        .map(|e| ResShowData {
            res: e.0,
            tags: if param.tags { Some(e.2) } else { None },
            ops: if param.ops { Some(e.1) } else { None },
        })
        .collect::<Vec<ResShowData>>();
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
    Ok(JsonData::data(json!({ "data": out,"total":count})))
}

#[derive(Debug, Serialize)]
pub struct ShowResItem {
    pub op: String,
    pub is_must: bool, //是否已经被管理的权限，即数据库里有权限的记录
}
#[derive(Debug, Serialize)]
pub struct ShowResData {
    pub res: String,
    pub ops: Vec<ShowResItem>,
    pub user_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ResAllParam {
    pub global_res: Option<bool>,
}

//资源授权
pub async fn rbac_all_res_list(res_tpl: &[ResTpl], param: ResAllParam) -> JsonResult<JsonData> {
    let res = res_tpl
        .iter()
        .filter(|e| match param.global_res {
            Some(g) => g != e.user,
            None => true,
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "data": res })))
}
