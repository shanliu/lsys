use lsys_core::db::Insert;
use lsys_core::{fluent_message, now_time, FluentMessage, RequestEnv};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::OnceLock};
use tokio::sync::mpsc::{self, Sender};
use tracing::{info, warn};

use crate::{
    dao::{
        op::OpInfo,
        res::ResInfo,
        result::{RbacError, RbacResult},
        role::{AccessResInfo, AccessRoleData, AccessRoleInfo, AccessRoleRow},
    },
    model::{
        RbacAuditDetailModel, RbacAuditDetailModelRef, RbacAuditModel, RbacAuditModelRef,
        RbacOpModel, RbacResModel,
    },
};

use super::RbacAccess;

//进行权限校验

pub struct AccessCheckRes<'t> {
    pub user_id: u64,              //资源用户ID
    pub app_id: u64,               //app_id >0 时,该用户 使用app
    pub res_type: &'t str,         //资源类型
    pub res_data: &'t str,         //资源数据
    pub op_key_data: Vec<&'t str>, //授权操作结构列表,不用&'t [&'t str],因为多层数组难以转换类型
}

impl<'t> AccessCheckRes<'t> {
    // 用户待验证资源
    pub fn user<'s: 't>(
        user_id: u64,
        res_type: &'s str,
        res_data: &'s str,
        op_key_data: Vec<&'s str>,
    ) -> AccessCheckRes<'s> {
        AccessCheckRes {
            res_type,
            app_id: 0,
            res_data,
            user_id,
            op_key_data,
        }
    }
    // 用户待验证资源
    pub fn user_empty_data<'s: 't>(
        user_id: u64,
        res_type: &'s str,
        ops: Vec<&'s str>,
    ) -> AccessCheckRes<'s> {
        Self::user(user_id, res_type, "", ops)
    }
    // 用户APP待验证资源
    pub fn user_app<'s: 't>(
        user_id: u64,
        app_id: u64,
        res_type: &'s str,
        res_data: &'s str,
        op_key_data: Vec<&'s str>,
    ) -> AccessCheckRes<'s> {
        AccessCheckRes {
            res_type,
            app_id,
            res_data,
            user_id,
            op_key_data,
        }
    }
    // 用户APP待验证资源
    pub fn user_app_empty_data<'s: 't>(
        user_id: u64,
        app_id: u64,
        res_type: &'s str,
        ops: Vec<&'s str>,
    ) -> AccessCheckRes<'s> {
        AccessCheckRes {
            res_type,
            app_id,
            res_data: "",
            user_id,
            op_key_data: ops,
        }
    }
    // 系统待验证资源
    pub fn system<'s: 't>(
        res_type: &'s str,
        res_data: &'s str,
        ops: Vec<&'s str>,
    ) -> AccessCheckRes<'s> {
        Self::user(0, res_type, res_data, ops)
    }
    // 系统待验证资源
    pub fn system_empty_data<'s: 't>(res_type: &'s str, ops: Vec<&'s str>) -> AccessCheckRes<'s> {
        Self::user(0, res_type, "", ops)
    }
}

//会话角色
#[derive(Clone)]
pub struct AccessSessionRole<'t> {
    pub role_key: &'t str,
    pub user_id: u64, //user_id >0 时,会话角色属于用户ID
    pub app_id: u64,  //app_id >0 时,该用户 使用app
}

#[derive(Default, Clone)]
pub struct AccessCheckEnv<'t> {
    pub user_id: u64, //0 为游客 或具体的访问用户id
    pub req_env: Option<&'t RequestEnv>,
    pub login_token_data: Option<&'t str>,
    //资源所属于用户跟访问用户的关系KEY数组，如公开角色,已登录角色,粉丝关系，指定应用关联等
    //该数据直接映射为对应角色
    pub session_role: Vec<AccessSessionRole<'t>>,
}

//授权失败结果
#[derive(Debug)]
pub struct AccessUnauthRes {
    pub user_id: u64,
    pub res_type: String,
    pub res_data: String,
    pub op_key: String,
    pub res_id: Option<u64>,
    pub op_id: Option<u64>,
    pub res_name: Option<String>,
    pub op_name: Option<String>,
    pub msg: FluentMessage,
}

//权限检测结果
struct AccessCheckItem<'t> {
    check_res_item: &'t AccessCheckRes<'t>,
    op_key: &'t str,
    res_detail: Option<&'t RbacResModel>,
    op_detail: Option<&'t RbacOpModel>,
    role_data: Vec<&'t AccessRoleRow>,
    is_self: bool,
    is_root: bool,
    is_role_excluce: bool,
    is_role_include: bool,
    is_role_all: bool,
    check_result: bool,
}

enum AccessRoleList {
    Root,
    RoleData(AccessRoleData),
}

impl RbacAccess {
    pub async fn check(
        &self,
        //请求检测环境数据
        env_data: &AccessCheckEnv<'_>,
        //待检测资源需要操作的列表
        check_res_data: &[AccessCheckRes<'_>],
    ) -> RbacResult<()> {
        if check_res_data.is_empty() {
            return Ok(());
        }
        //把check_res_data转为数据库记录
        //user_id+res_type+res_data => yaf_rbac_res
        //user_id+op_key_data => yaf_rbac_op
        let res_info = check_res_data
            .iter()
            .map(|e| ResInfo {
                res_type: e.res_type,
                res_data: e.res_data,
                user_id: e.user_id,
                app_id: e.app_id,
            })
            .collect::<Vec<_>>();
        let res_list = self.res.cache().find_vec_by_info(&res_info).await?;
        let op_info = check_res_data
            .iter()
            .flat_map(|e| {
                e.op_key_data
                    .iter()
                    .map(|w| OpInfo {
                        op_key: w,
                        app_id: e.app_id,
                        user_id: e.user_id,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let op_list = self.op.cache().find_vec_by_info(&op_info).await?;
        let mut check_data = vec![];

        let access_role_list = if !self.is_root(&env_data.user_id) {
            let mut tmp_check = HashMap::new();
            for item in check_res_data {
                tmp_check
                    .entry((item.user_id, item.app_id))
                    .or_insert_with(Vec::new)
                    .push(item);
            }
            let res_check = &tmp_check
                .iter()
                .map(|((tu_id, tapp_id), res_tmp)| {
                    let res_data = res_tmp
                        .iter()
                        .flat_map(|r| {
                            let op_data = r
                                .op_key_data
                                .iter()
                                .filter_map(|o| {
                                    op_list
                                        .iter()
                                        .find(|r| {
                                            r.0.op_key == *o
                                                && *tu_id == r.0.user_id
                                                && *tapp_id == r.0.app_id
                                        })
                                        .and_then(|r| r.1.as_ref())
                                })
                                .collect::<Vec<_>>();
                            res_list
                                .iter()
                                .find(|s| {
                                    s.0.res_data == r.res_data
                                        && s.0.res_type == r.res_type
                                        && s.0.user_id == r.user_id
                                        && s.0.app_id == r.app_id
                                })
                                .and_then(|s| s.1.as_ref())
                                .map(move |s| (s, op_data))
                        })
                        .collect::<Vec<_>>();
                    AccessResInfo {
                        user_id: *tu_id,
                        app_id: *tapp_id,
                        res_data,
                    }
                })
                .collect::<Vec<_>>();
            let role_check = env_data
                .session_role
                .iter()
                .map(|e| AccessRoleInfo {
                    user_id: e.user_id,
                    app_id: e.app_id,
                    role_key: e.role_key,
                })
                .collect::<Vec<_>>();
            //根据访问用户 访问资源 会话角色 获取用于权限校验的相关角色列表
            AccessRoleList::RoleData(
                self.role
                    .cache()
                    .find_access_row(env_data.user_id, res_check, &role_check)
                    .await?,
            )
        } else {
            AccessRoleList::Root
        };

        let sys_all = match access_role_list {
            AccessRoleList::Root => vec![],
            AccessRoleList::RoleData(ref role_list) => role_list.get_system_all_role(),
        };
        for res_item in check_res_data {
            let res_detail = res_list
                .iter()
                .find(|e| {
                    e.0.res_data == res_item.res_data
                        && e.0.res_type == res_item.res_type
                        && e.0.user_id == res_item.user_id
                        && e.0.app_id == res_item.app_id
                })
                .and_then(|e| e.1.as_ref());

            let user_all = OnceLock::new();

            for op_item in res_item.op_key_data.iter() {
                let op_detail = op_list
                    .iter()
                    .find(|e| {
                        e.0.op_key == *op_item
                            && e.0.user_id == res_item.user_id
                            && e.0.app_id == res_item.app_id
                    })
                    .and_then(|e| e.1.as_ref());

                match access_role_list {
                    AccessRoleList::Root => {
                        //pass
                        check_data.push(AccessCheckItem {
                            check_res_item: res_item,
                            op_key: op_item,
                            role_data: vec![],
                            res_detail,
                            op_detail,
                            is_self: false,
                            is_root: true,
                            is_role_all: false,
                            is_role_excluce: false,
                            is_role_include: false,
                            check_result: true,
                        });
                        continue;
                    }
                    AccessRoleList::RoleData(ref role_list) => {
                        let sys_excluce =
                            if let (Some(res_val), Some(op_val)) = (res_detail, op_detail) {
                                role_list.get_system_exclude_role(res_val.id, op_val.id)
                            } else {
                                vec![]
                            };
                        //系统屏蔽
                        if !sys_excluce.is_empty() {
                            //bad
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: sys_excluce,
                                res_detail,
                                op_detail,
                                is_self: false,
                                is_root: false,
                                is_role_all: false,
                                is_role_excluce: true,
                                is_role_include: false,
                                check_result: false,
                            });

                            continue;
                        }
                        //系统允许全部
                        if !sys_all.is_empty() {
                            //pass
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: sys_all.clone(),
                                res_detail,
                                op_detail,
                                is_self: false,
                                is_root: false,
                                is_role_all: true,
                                is_role_excluce: false,
                                is_role_include: false,
                                check_result: true,
                            });
                            continue;
                        }

                        let sys_include =
                            if let (Some(res_val), Some(op_val)) = (res_detail, op_detail) {
                                role_list.get_system_exclude_role(res_val.id, op_val.id)
                            } else {
                                vec![]
                            };
                        //系统允许部分
                        if !sys_include.is_empty() {
                            //pass
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: sys_include,
                                res_detail,
                                op_detail,
                                is_self: false,
                                is_root: false,
                                is_role_all: false,
                                is_role_excluce: false,
                                is_role_include: true,
                                check_result: true,
                            });
                            continue;
                        }
                        //自身资源
                        if env_data.user_id == res_item.user_id {
                            //pass
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: vec![],
                                res_detail,
                                op_detail,
                                is_self: true,
                                is_root: false,
                                is_role_all: false,
                                is_role_excluce: false,
                                is_role_include: false,
                                check_result: true,
                            });
                            continue;
                        }
                        //用户屏蔽
                        let user_excluce = if let (Some(res_val), Some(op_val)) =
                            (res_detail, op_detail)
                        {
                            role_list.get_user_exclude_role(res_item.user_id, res_val.id, op_val.id)
                        } else {
                            vec![]
                        };

                        if !user_excluce.is_empty() {
                            //bad
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: user_excluce,
                                res_detail,
                                op_detail,
                                is_self: false,
                                is_root: false,
                                is_role_all: false,
                                is_role_excluce: true,
                                is_role_include: false,
                                check_result: false,
                            });

                            continue;
                        }
                        //用户允许全部
                        if !user_all
                            .get_or_init(|| role_list.get_user_all_role(res_item.user_id))
                            .is_empty()
                        {
                            //pass
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: user_all
                                    .get()
                                    .map(|e| e.to_owned())
                                    .to_owned()
                                    .unwrap_or_default(),
                                res_detail,
                                op_detail,
                                is_self: false,
                                is_root: false,
                                is_role_all: true,
                                is_role_excluce: false,
                                is_role_include: false,
                                check_result: true,
                            });
                            continue;
                        }
                        //用户允许部分
                        let user_include = if let (Some(res_val), Some(op_val)) =
                            (res_detail, op_detail)
                        {
                            role_list.get_user_include_role(res_item.user_id, res_val.id, op_val.id)
                        } else {
                            vec![]
                        };

                        if !user_include.is_empty() {
                            //pass
                            check_data.push(AccessCheckItem {
                                check_res_item: res_item,
                                op_key: op_item,
                                role_data: user_include,
                                res_detail,
                                op_detail,
                                is_self: false,
                                is_root: false,
                                is_role_all: false,
                                is_role_excluce: false,
                                is_role_include: true,
                                check_result: true,
                            });
                            continue;
                        }
                        //无任何匹配的角色
                        //bad
                        check_data.push(AccessCheckItem {
                            check_res_item: res_item,
                            op_key: op_item,
                            role_data: vec![],
                            res_detail,
                            op_detail,
                            is_self: false,
                            is_root: false,
                            is_role_all: false,
                            is_role_excluce: false,
                            is_role_include: false,
                            check_result: false,
                        });
                    }
                }
            }
        }
        let bad_item=check_data.iter().flat_map(|check_item|{
            if check_item.check_result {
                None                
            }else{
                Some(AccessUnauthRes{
                    user_id: check_item.check_res_item.user_id,
                    res_type:   check_item.check_res_item.res_type.to_owned(),
                    res_data:  check_item.check_res_item.res_data.to_owned(),
                    op_key: check_item.op_key.to_owned(),
                    res_id: check_item.res_detail.map(|e|e.id),
                    op_id: check_item.op_detail.map(|e|e.id),
                    res_name:check_item.res_detail.map(|e|e.res_name.to_owned()),
                    op_name: check_item.op_detail.map(|e|e.op_name.to_owned()),
                    msg:if check_item.is_role_excluce {
                        fluent_message!( "rbac-access-block",{//主动禁止
                            "res_name":check_item.res_detail.as_ref().map(|e|e.res_name.as_str()).unwrap_or(check_item.check_res_item.res_type),
                            "op_name":check_item.op_detail.as_ref().map(|e|e.op_name.as_str()).unwrap_or(check_item.op_key),
                            "user_id":check_item.check_res_item.user_id,
                        })
                        }else{ fluent_message!( "rbac-access-unauth",{//未授权
                            "res_name":check_item.res_detail.as_ref().map(|e|e.res_name.as_str()).unwrap_or(check_item.check_res_item.res_type),
                            "op_name":check_item.op_detail.as_ref().map(|e|e.op_name.as_str()).unwrap_or(check_item.op_key),
                            "user_id":check_item.check_res_item.user_id,
                        })},
                })
            }
        }).collect::<Vec<_>>();
        self.check_add_audit(env_data, bad_item.is_empty(), &check_data)
            .await;
        if !bad_item.is_empty() {
            return Err(RbacError::Check(bad_item));
        }
        Ok(())
    }
    pub async fn list_check(
        &self,
        //请求检测环境数据
        env_data: &AccessCheckEnv<'_>,
        //资源所属于用户跟访问用户的关系KEY数组，如粉丝关系，指定应用关联等
        //该数据直接映射为对应角色
        // session_role_data: &[AccessSessionRole<'_>],
        //待检测资源需要操作的列表
        check_res_list: &[&[AccessCheckRes<'_>]],
    ) -> RbacResult<()> {
        let mut bad_data = vec![];
        for check_res_data in check_res_list {
            match self.check(env_data, check_res_data).await {
                //优化:res 可以批量查询后在合并验证
                Ok(()) => return Ok(()),
                Err(err) => match err {
                    RbacError::Check(mut bad) => bad_data.append(&mut bad),
                    err => return Err(err),
                },
            }
        }
        Err(RbacError::Check(bad_data))
    }
}

pub(crate) struct AuditItem {
    user_id: u64,
    role_key_data: String,
    check_result: i8,
    token_data: String,
    user_ip: String,
    device_id: String,
    device_name: String,
    request_id: String,
    add_time: u64,
    detail: Vec<AuditItemDetail>,
}
#[derive(Serialize, Deserialize)]
pub struct AuditItemRole {
    pub role_id: u64,
    pub role_name: String,
    pub role_key: String,
    pub perm_id: u64,
    pub access_timeout: u64,
    pub access_user_id: u64,
}

struct AuditItemDetail {
    res_type: String,
    res_data: String,
    res_user_id: u64,
    op_key: String,
    res_id: u64,
    op_id: u64,
    check_result: i8,
    is_self: i8,
    is_root: i8,
    is_role_excluce: i8,
    is_role_include: i8,
    is_role_all: i8,
    role_data: String,
}

// check audit
impl RbacAccess {
    async fn check_add_audit(
        &self,
        env_data: &AccessCheckEnv<'_>,
        check_result: bool,
        // session_role_data: &[AccessSessionRole<'_>],
        detail_data: &[AccessCheckItem<'_>],
    ) {
        let role_key_data = json!(env_data
            .session_role
            .iter()
            .map(|e| {
                json!({
                    "key":e.role_key,
                    "user_id":e.user_id,
                })
            })
            .collect::<Vec<_>>())
        .to_string();
        let add_time = now_time().unwrap_or_default();
        let user_ip = env_data
            .req_env
            .as_ref()
            .map(|e| {
                e.request_ip
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(40)
            .collect();
        let request_id = env_data
            .req_env
            .as_ref()
            .map(|e| {
                e.request_id
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(32)
            .collect();
        let device_name = env_data
            .req_env
            .as_ref()
            .map(|e| {
                e.request_user_agent
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(254)
            .collect();
        let device_id = env_data
            .req_env
            .as_ref()
            .map(|e| {
                e.device_id
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .chars()
            .take(64)
            .collect();
        let send_data = AuditItem {
            user_id: env_data.user_id,
            role_key_data,
            check_result: if check_result { 1 } else { 0 },
            token_data: env_data
                .login_token_data
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_default(),
            user_ip,
            device_id,
            device_name,
            request_id,
            add_time,
            detail: detail_data
                .iter()
                .map(|e| {
                    AuditItemDetail {
                        res_type: e.check_res_item.res_type.to_owned(),
                        res_data: e.check_res_item.res_data.to_owned(),
                        res_user_id: e.check_res_item.user_id,
                        op_key: e.op_key.to_owned(),
                        res_id: e.res_detail.map(|e| e.id).unwrap_or_default(),
                        op_id: e.op_detail.map(|e| e.id).unwrap_or_default(),
                        check_result: if e.check_result { 1 } else { 0 },
                        is_self: if e.is_self { 1 } else { 0 },
                        is_root: if e.is_root { 1 } else { 0 },
                        is_role_excluce: if e.is_role_excluce { 1 } else { 0 },
                        is_role_include: if e.is_role_include { 1 } else { 0 },
                        is_role_all: if e.is_role_all { 1 } else { 0 },
                        role_data: json!(e
                            .role_data
                            .iter()
                            .map(|e| {
                                AuditItemRole {
                                    role_id: e.role.id,
                                    role_name: e.role.role_name.to_owned(),
                                    role_key: e.role.role_key.to_owned(),
                                    perm_id: e.perm_id,
                                    access_timeout: e.access_timeout,
                                    access_user_id: e.access_user_id,
                                }
                            })
                            .collect::<Vec<_>>())
                        .to_string(),
                    }
                })
                .collect::<Vec<_>>(),
        };
        match self.audit_sender {
            Some(ref sender) => {
                if let Err(err) = sender.send(send_data).await {
                    warn!("async add audit fail:{}", err);
                    Self::audit_add(&self.db, err.0).await;
                }
            }
            None => {
                Self::audit_add(&self.db, send_data).await;
            }
        }
    }
    async fn audit_add(db: &sqlx::Pool<sqlx::MySql>, msg: AuditItem) {
        match db.begin().await {
            Ok(mut db_tran) => {
                let vdata = lsys_core::model_option_set!(RbacAuditModelRef,{
                    user_id:msg.user_id,
                    role_key_data:msg.role_key_data,
                    check_result:msg.check_result,
                    token_data:msg.token_data,
                    user_ip:msg.user_ip,
                    device_id:msg.device_id,
                    device_name:msg.device_name,
                    request_id:msg.request_id,
                    add_time:msg.add_time,
                });
                let rbac_audit_id = match Insert::<RbacAuditModel, _>::new(vdata)
                    .execute(&mut *db_tran)
                    .await
                {
                    Ok(id) => id.last_insert_id(),
                    Err(err) => {
                        let _ = db_tran.rollback().await;
                        warn!("add audit fail,on add:{err}");
                        return;
                    }
                };
                if !msg.detail.is_empty() {
                    let mut dvdata = Vec::with_capacity(msg.detail.len());
                    for tmp in msg.detail.iter() {
                        dvdata.push(lsys_core::model_option_set!(RbacAuditDetailModelRef,{
                            res_type:tmp.res_type,
                            res_data:tmp.res_data,
                            res_user_id:tmp.res_user_id,
                            op_key:tmp.op_key,
                            rbac_audit_id:rbac_audit_id,
                            check_result:tmp.check_result,
                            add_time:msg.add_time,
                            res_id:tmp.res_id,
                            op_id:tmp.op_id,
                            role_data:tmp.role_data,
                            is_role_all:tmp.is_role_all,
                            is_role_include:tmp.is_role_include,
                            is_role_excluce:tmp.is_role_excluce,
                            is_root:tmp.is_root,
                            is_self:tmp.is_self,
                        }));
                    }
                    if let Err(err) =
                        Insert::<RbacAuditDetailModel, _>::new_vec(dvdata)
                            .execute(&mut *db_tran)
                            .await
                    {
                        let _ = db_tran.rollback().await;
                        warn!("add audit fail,on add detail:{err}");
                        return;
                    };
                }
                if let Err(err) = db_tran.commit().await {
                    warn!("add audit fail,on commit:{err}");
                };
            }
            Err(err) => {
                warn!("add audit fail,on tran:{err}");
            }
        }
    }
    pub(crate) fn listen_audit(
        db: sqlx::Pool<sqlx::MySql>,
        limit: usize,
    ) -> Option<Sender<AuditItem>> {
        if limit == 0 {
            return None;
        }
        let (tx, mut rx) = mpsc::channel::<AuditItem>(limit);
        info!("rbac audit listen start");
        tokio::task::spawn(async move {
            info!("rbac audit add start");
            while let Some(msg) = rx.recv().await {
                info!("rbac audit listen add:{}", msg.request_id);
                Self::audit_add(&db, msg).await;
            }
            info!("rbac audit add end");
        });
        info!("rbac audit listen end");
        Some(tx)
    }
}
