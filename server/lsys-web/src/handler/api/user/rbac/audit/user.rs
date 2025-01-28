use crate::common::{JsonResult, PageParam, UserAuthQueryDao};

use lsys_rbac::{
    dao::{AccessPublicResUserData, AccessResRoleRow, AccessResUserData, AccessResUserRow},
    model::RbacRoleUserRange,
};

pub struct RbacUserFromResParam {
    user_id: u64,     //资源用户ID
    res_type: String, //资源类型
    res_data: String, //资源数据
    op_key: String,   //授权操作结构列表
}

//1 得到指定资源的授权详细
pub async fn res_user_from_res(
    param: &RbacUserFromResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(AccessResUserData, AccessPublicResUserData)> {
    let user_set_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_data_from_res(
            param.user_id,
            &param.res_type,
            &param.res_data,
            &param.op_key,
        )
        .await?;
    let pub_set_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_data_from_public(param.user_id)
        .await?;
    Ok((user_set_data, pub_set_data))
}
pub struct RbacResRoleFromResParam {
    pub user_id: u64,     //资源用户ID
    pub res_type: String, //资源类型
    pub res_data: String, //资源数据
    pub op_key: String,   //授权操作结构列表
    pub res_range_exclude: bool,
    pub res_range_any: bool,
    pub res_range_include: bool,
    pub is_system: bool,
    pub is_self: bool,
    pub user_range: Option<Vec<i8>>,
    pub page: Option<PageParam>,
}

//获取非特定用户授权的角色列表
pub async fn res_role_data_from_res(
    param: &RbacResRoleFromResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<AccessResRoleRow>, i64)> {
    let user_range = match &param.user_range {
        Some(dat) => {
            let mut out = vec![];
            for tmp in dat {
                out.push(RbacRoleUserRange::try_from(*tmp)?);
            }
            Some(out)
        }
        None => None,
    };

    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_role_list_from_res(
            param.user_id,
            &param.res_type,
            &param.res_data,
            &param.op_key,
            param.res_range_exclude,
            param.res_range_any,
            param.res_range_include,
            param.is_system,
            param.is_self,
            user_range.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let res_count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_role_count_from_res(
            param.user_id,
            &param.res_type,
            &param.res_data,
            &param.op_key,
            param.res_range_exclude,
            param.res_range_any,
            param.res_range_include,
            param.is_system,
            param.is_self,
            user_range.as_deref(),
        )
        .await?;
    Ok((res_data, res_count))
}

pub struct RbacResUserDataFromResParam {
    user_id: u64,     //资源用户ID
    res_type: String, //资源类型
    res_data: String, //资源数据
    op_key: String,   //授权操作结构列表
    res_range_exclude: bool,
    res_range_any: bool,
    res_range_include: bool,
    is_system: bool,
    is_self: bool,
    pub page: Option<PageParam>,
}
//获取特定用户授权列表
pub async fn res_user_data_from_res(
    param: &RbacResUserDataFromResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<AccessResUserRow>, i64)> {
    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_list_from_res(
            param.user_id,
            &param.res_type,
            &param.res_data,
            &param.op_key,
            param.res_range_exclude,
            param.res_range_any,
            param.res_range_include,
            param.is_system,
            param.is_self,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let res_count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_count_from_res(
            param.user_id,
            &param.res_type,
            &param.res_data,
            &param.op_key,
            param.res_range_exclude,
            param.res_range_any,
            param.res_range_include,
            param.is_system,
            param.is_self,
        )
        .await?;
    Ok((res_data, res_count))
}
