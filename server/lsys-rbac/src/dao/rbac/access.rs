use std::{sync::Arc, vec};

use lsys_core::fluent_message;
use tracing::{debug, info};

use crate::model::{RbacRoleModel, RbacRoleOpPositivity};

use super::{
    res::RbacRes,
    role::{RbacRole, RoleRelationKey},
    RbacResData, ResKey, UserRbacError, UserRbacResult, ROLE_PRIORITY_MAX, ROLE_PRIORITY_MIN,
    ROLE_PRIORITY_NONE,
};

//待检查权限角色记录
#[derive(Clone, Debug)]
pub enum RoleCheckRow {
    ModelRole {
        role: Option<(RbacRoleModel, RbacRoleOpPositivity)>,
        res_op_id: u64,
    },
    InnerRole {
        res_op_id: u64,
        positivity: RbacRoleOpPositivity,
        priority: i8,
    },
}
impl ToString for RoleCheckRow {
    fn to_string(&self) -> String {
        match self {
            RoleCheckRow::ModelRole { role, res_op_id } => match role {
                Some((role, positivity)) => {
                    format!(
                        "res:{} model role[{}]:{} priority:{}",
                        res_op_id,
                        role.id,
                        if *positivity == RbacRoleOpPositivity::Allow {
                            "allow"
                        } else {
                            "deny"
                        },
                        role.priority,
                    )
                }
                None => "model role not find".to_string(),
            },
            RoleCheckRow::InnerRole {
                res_op_id,
                positivity,
                priority,
            } => {
                format!(
                    "res:{} inner role:{} priority:{}",
                    res_op_id,
                    if *positivity == RbacRoleOpPositivity::Allow {
                        "allow"
                    } else {
                        "deny"
                    },
                    priority,
                )
            }
        }
    }
}

impl RoleCheckRow {
    fn res_op_id(&self) -> u64 {
        match self {
            RoleCheckRow::ModelRole { role: _, res_op_id } => *res_op_id,
            RoleCheckRow::InnerRole {
                res_op_id,
                positivity: _,
                priority: _,
            } => *res_op_id,
        }
    }
    fn priority(&self) -> i8 {
        match self {
            RoleCheckRow::ModelRole { role, res_op_id: _ } => role
                .as_ref()
                .map(|e| e.0.priority)
                .unwrap_or(ROLE_PRIORITY_NONE),
            RoleCheckRow::InnerRole {
                res_op_id: _,
                positivity: _,
                priority,
            } => *priority,
        }
    }
    pub fn is_pass(&self) -> bool {
        match self {
            RoleCheckRow::ModelRole { role, res_op_id: _ } => match role {
                Some((_, positivity)) => *positivity == RbacRoleOpPositivity::Allow,
                None => false,
            },
            RoleCheckRow::InnerRole {
                res_op_id: _,
                positivity,
                priority: _,
            } => *positivity == RbacRoleOpPositivity::Allow,
        }
    }
}

//待检查权限角色数据[角色记录集]
#[derive(Clone, Debug)]
pub struct RoleCheckData(Vec<RoleCheckRow>);
impl RoleCheckData {
    pub fn new(row: Vec<RoleCheckRow>) -> Self {
        Self(row)
    }
    pub fn match_role(&self, find_res_op_id: u64) -> Option<&RoleCheckRow> {
        self.0.iter().find(|&tmp| tmp.res_op_id() == find_res_op_id)
    }
    pub fn merge(&self, other: RoleCheckData) -> Self {
        let mut out = Vec::with_capacity(if self.0.len() > other.0.len() {
            self.0.len()
        } else {
            other.0.len()
        });
        for tmp1 in self.0.iter() {
            let mut add = false;
            for tmp2 in other.0.iter() {
                if tmp1.res_op_id() == tmp2.res_op_id() && tmp2.priority() > tmp1.priority() {
                    add = true;
                    out.push(tmp2.to_owned())
                }
            }
            if !add {
                out.push(tmp1.to_owned())
            }
        }
        for tmp2 in other.0.iter() {
            if !out.iter().any(|e| e.res_op_id() == tmp2.res_op_id()) {
                out.push(tmp2.to_owned())
            }
        }
        Self(out)
    }
}

//待检查授权资源
#[derive(Clone, Debug)]
pub struct AccessRes {
    pub res: String,             //资源KEY
    pub user_id: u64,            //资源用户ID
    pub ops: Vec<String>,        //必须的权限
    pub option_ops: Vec<String>, //可选的权限,当未添加时不进行权限验证
}

impl AccessRes {
    // 用户待验证资源
    pub fn user(user_id: u64, name: &str, ops: &[&str], option_ops: &[&str]) -> Self {
        Self {
            res: name.to_string(),
            user_id,
            ops: ops.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
            option_ops: option_ops.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
        }
    }
    // 系统待验证资源
    pub fn system(name: &str, ops: &[&str], option_ops: &[&str]) -> Self {
        Self::user(0, name, ops, option_ops)
    }
}

//需检查权资源
#[derive(Clone, Debug)]
pub struct CheckRes {
    pub res: String,      //资源KEY
    pub user_id: u64,     //资源用户ID
    pub ops: Vec<String>, //授权操作结构列表
}

//需要验证但未被管理的资源
#[derive(Clone)]
pub struct CheckResData {
    pub find_res: Option<RbacResData>, //查询到的资源
    pub check_res: CheckRes,           //需检查权资源
}

//系统角色实现
//用代码方式实现角色
pub trait SystemRoleCheckData: Sync + Send {
    /// 过滤掉无需检查授权资源 [为实现跳过一些未资源表注册资源检查，如超级用户访问时未注册也需有权限]
    /// items 为外部传入待检测授权资源
    /// 返回需检查授权资源
    fn filter_check_res(&self, user_id: u64, items: &[CheckResData]) -> Vec<CheckResData>;
    /// 根据需要验证资源返回角色检测数据，跟数据库中查找到记录合并后进行权限校验
    /// 如超级用户访问赋予通过权限，自己访问自己资源赋予可操作权限
    fn role_check_data(&self, user_id: u64, check_vec: &[RbacResData]) -> RoleCheckData;
}

//系统内置角色
pub struct SystemRole {
    root_user_id: Vec<u64>,
    self_res: bool,
}
impl SystemRole {
    pub fn new(
        self_res: bool,         //当资源为自身时是否直接授权
        root_user_id: Vec<u64>, //超级用户ID,在此列表的用户直接授权
    ) -> Self {
        Self {
            root_user_id,
            self_res,
        }
    }
}
impl SystemRoleCheckData for SystemRole {
    fn role_check_data(&self, user_id: u64, check_vec: &[RbacResData]) -> RoleCheckData {
        let mut data = vec![];
        if self.root_user_id.iter().any(|e| *e == user_id) {
            let mut tmp =
                Vec::with_capacity(check_vec.iter().fold(0, |acc, res| acc + res.ops.len()));
            //超级用户，返回最高权限且通过检查的权限检测结果
            for check_item in check_vec.iter() {
                for res_op in check_item.ops.iter() {
                    tmp.push(RoleCheckRow::InnerRole {
                        res_op_id: res_op.id,
                        positivity: RbacRoleOpPositivity::Allow,
                        priority: ROLE_PRIORITY_MAX,
                    });
                }
            }
            data.extend(tmp);
        }
        if self.self_res {
            let mut tmp =
                Vec::with_capacity(check_vec.iter().fold(0, |acc, res| acc + res.ops.len()));
            //自身资源，返回最底权限且通过检查的权限检测结果
            //外部有其他权限规则时覆盖这个默认规则
            for check_item in check_vec.iter() {
                if user_id == check_item.res.user_id {
                    for res_op in check_item.ops.iter() {
                        tmp.push(RoleCheckRow::InnerRole {
                            res_op_id: res_op.id,
                            positivity: RbacRoleOpPositivity::Allow,
                            priority: ROLE_PRIORITY_MIN,
                        });
                    }
                }
            }
            data.extend(tmp);
        };
        RoleCheckData(data)
    }
    fn filter_check_res(&self, user_id: u64, items: &[CheckResData]) -> Vec<CheckResData> {
        if self.root_user_id.iter().any(|e| *e == user_id) {
            //超级用户访问，不检查权限是否在数据库里有资源记录
            return vec![];
        }
        if self.self_res {
            //对于访问自身资源，不检查是否在在数据库有资源记录
            return items
                .iter()
                .filter(|e| e.check_res.user_id != user_id)
                .map(|e| e.to_owned())
                .collect::<Vec<CheckResData>>();
        }
        items.to_vec()
    }
}

/// RBAC对外结构
pub struct RbacAccess {
    res: Arc<RbacRes>,
    role: Arc<RbacRole>,
    system_role: Option<Box<dyn SystemRoleCheckData>>,
    use_cache: bool,
}

impl RbacAccess {
    pub fn new(
        res: Arc<RbacRes>,
        role: Arc<RbacRole>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        use_cache: bool,
    ) -> Self {
        RbacAccess {
            // fluent,
            res,
            role,
            system_role,
            use_cache,
        }
    }
    pub async fn list_check(
        &self,
        //0 为游客 或具体的访问用户id
        user_id: u64,
        //资源所属于用户跟访问用户的关系KEY数组，如粉丝关系，指定应用关联等
        //该数据直接映射为对应角色
        relation_key_roles: &[RoleRelationKey],
        //待检测资源需要操作的列表
        //结果二维数组
        //  第一层为 || 或关系 该层的任意一个满足条件为授权通过
        //  第二层为 && 并关系 该层所有条件都必须满足为授权通过
        //可通过上面的二维数组可用组合出各种所需要的验证场景
        check_vec: &[Vec<AccessRes>],
    ) -> UserRbacResult<()> {
        let mut check_data = Vec::with_capacity(check_vec.len());
        for tmp in check_vec {
            check_data.push(self.find_option_res(tmp).await?);
        }
        if check_data.is_empty() {
            return Ok(());
        }
        let mut bad_data = vec![];
        for check_res in check_data {
            match self
                .check_res(user_id, relation_key_roles, &check_res)
                .await
            {
                Ok(()) => return Ok(()),
                Err(err) => match err {
                    UserRbacError::Check(mut bad) => bad_data.append(&mut bad),
                    err => return Err(err),
                },
            }
        }
        Err(UserRbacError::Check(bad_data))
    }
    pub async fn check(
        &self,
        //0 为游客 或具体的访问用户id
        user_id: u64,
        //资源所属于用户跟访问用户的关系KEY数组，如粉丝关系，指定应用关联等
        //该数据直接映射为对应角色
        relation_key_roles: &[RoleRelationKey],
        //待检测资源需要操作的列表
        check_vec: &[AccessRes],
    ) -> UserRbacResult<()> {
        let check_data = self.find_option_res(check_vec).await?;
        self.check_res(user_id, relation_key_roles, &check_data)
            .await
    }
    /// 待检测资源需要操作的列表
    /// 忽略不存在的可选资源
    pub async fn find_option_res(
        &self,
        check_option_vec: &[AccessRes],
    ) -> UserRbacResult<Vec<CheckResData>> {
        let check_data_all = self
            .find_res(
                &check_option_vec
                    .iter()
                    .map(|e| CheckRes {
                        res: e.res.clone(),
                        user_id: e.user_id,
                        ops: e
                            .ops
                            .clone()
                            .into_iter()
                            .chain(e.option_ops.clone())
                            .collect::<Vec<_>>(),
                    })
                    .collect::<Vec<_>>(),
            )
            .await?;
        let check_data = check_data_all
            .into_iter()
            .filter_map(|e| {
                for t in check_option_vec.iter() {
                    if t.res == e.check_res.res && t.user_id == e.check_res.user_id {
                        match e.find_res {
                            Some(res) => {
                                let ops = t
                                    .option_ops
                                    .clone()
                                    .into_iter()
                                    .filter_map(|e| {
                                        if res.ops.iter().any(|r| r.op_key == e) {
                                            Some(e)
                                        } else {
                                            None
                                        }
                                    })
                                    .chain(t.ops.clone())
                                    .collect::<Vec<_>>();
                                return Some(CheckResData {
                                    find_res: Some(res),
                                    check_res: CheckRes {
                                        res: e.check_res.res,
                                        user_id: e.check_res.user_id,
                                        ops,
                                    },
                                });
                            }
                            None => {
                                if t.ops.is_empty() {
                                    return None;
                                }
                            }
                        }
                        break;
                    }
                }
                Some(e) //原样返回
            })
            .collect::<Vec<_>>();
        Ok(check_data)
    }
    /// 待检测资源需要操作的列表
    pub async fn find_res(&self, check_res: &[CheckRes]) -> UserRbacResult<Vec<CheckResData>> {
        let res_list = check_res
            .iter()
            .map(|e| ResKey {
                res_key: e.res.clone(),
                user_id: e.user_id,
            })
            .collect::<Vec<ResKey>>();
        if res_list.is_empty() {
            return Ok(vec![]);
        }
        //需要验证且数据库中存记录的资源
        let res_vec = if self.use_cache {
            self.res.cache().find_by_keys(&res_list).await
        } else {
            self.res.find_by_keys(&res_list).await
        }?
        .into_values()
        .flatten()
        .map(|mut res_data| {
            if let Some(acc_res) = check_res.iter().find(|e| e.res == res_data.res.res_key) {
                res_data.ops = res_data
                    .ops
                    .into_iter()
                    .filter(|e| acc_res.ops.iter().any(|ee| *ee == e.op_key))
                    .collect::<Vec<_>>();
            } else {
                res_data.ops = vec![];
            }
            res_data
        })
        .collect::<Vec<_>>();
        let mut out = Vec::with_capacity(check_res.len());
        //需要验证且不存在数据库记录的资源
        for acc_res in check_res.iter() {
            let mut find_res = false;
            for res in res_vec.iter() {
                //res 已存在在数据库的资源
                if res.res.res_key == acc_res.res {
                    find_res = true;
                    let mut bad_op_vec = vec![];
                    //acc_res 需要检测的资源
                    for oe in acc_res.ops.iter() {
                        if !res.ops.iter().any(|t| t.op_key == *oe) {
                            bad_op_vec.push(oe.to_owned());
                        }
                    }
                    out.push(CheckResData {
                        find_res: Some(res.to_owned()),
                        check_res: CheckRes {
                            res: acc_res.res.clone(),
                            user_id: acc_res.user_id,
                            ops: bad_op_vec,
                        },
                    });
                    break;
                }
            }
            if !find_res {
                out.push(CheckResData {
                    find_res: None,
                    check_res: acc_res.to_owned(),
                });
            }
        }
        Ok(out)
    }
    /// 校验指定资源访问权限
    pub async fn check_res(
        &self,
        //0 为游客 或具体的访问用户id
        user_id: u64,
        //资源所属于用户跟访问用户的关系KEY数组，如粉丝关系，指定应用关联等
        //该数据直接映射为对应角色
        relation_key_roles: &[RoleRelationKey],
        //待检测资源需要操作的列表
        res_data: &[CheckResData],
    ) -> UserRbacResult<()> {
        let tmp = if let Some(ref sys_role) = self.system_role {
            //系统角色过滤需要验证但不存在数据库记录的资源
            sys_role.filter_check_res(user_id, res_data)
        } else {
            vec![]
        };
        let res_data = if self.system_role.is_some() {
            &tmp
        } else {
            res_data
        };
        if res_data.is_empty() {
            //需要验证资源为空,返回授权通过
            return Ok(());
        }
        let mut res_vec = vec![];
        let mut bad_tmp = vec![];

        //遍历待检测权限资源,收集是否存在需要授权,但数据库中缺失对应记录的资源
        for tmp in res_data {
            match tmp.find_res {
                Some(ref res) => {
                    for otmp in &tmp.check_res.ops {
                        if !res.ops.iter().any(|e| e.op_key == *otmp) {
                            debug!(
                                "user {} acces, res[{}] {}:{}  op not find :{}",
                                user_id,
                                res.res.id,
                                &tmp.check_res.res,
                                tmp.check_res.user_id,
                                otmp
                            );
                            bad_tmp.push((
                                tmp.check_res.res.clone(),
                                fluent_message!("rbac-access-check-res-empty",{
                                   // "res_id":res.res.id,
                                    "res":tmp.check_res.res,
                                    "op":otmp,
                                    "user_id":tmp.check_res.user_id,
                                    "view_user_id":user_id
                                }),
                                //"Authorization not find user [{$user_id}] in res[{$res}:{$res_id}] on op {$op} [{$view_user}]"
                            ));
                        }
                    }
                    res_vec.push(res.to_owned())
                }
                None => {
                    for tmp_op in &tmp.check_res.ops {
                        debug!(
                            "user {} acces, res not find {}:{} op :{}",
                            user_id, &tmp.check_res.res, tmp.check_res.user_id, tmp_op
                        );
                        bad_tmp.push((
                            tmp.check_res.res.clone(),
                            fluent_message!("rbac-access-check-res-empty",{
                                    "res":tmp.check_res.res,
                                    "op":tmp_op,
                                    "user_id":tmp.check_res.user_id,
                                    "view_user":user_id
                            }),
                        )); // "Authorization not find  user [{$user_id}] res [{$res}] on op {$op} [{$view_user}]"
                    }
                }
            };
        }
        //存在需要授权,但数据库里没对应记录的资源,返回授权失败
        if !bad_tmp.is_empty() {
            return Err(UserRbacError::Check(bad_tmp));
        }
        if res_vec.is_empty() {
            return Ok(());
        }

        //获取内置角色
        let mut role_data = if let Some(ref sys_role) = self.system_role {
            sys_role.role_check_data(user_id, &res_vec)
        } else {
            RoleCheckData::new(vec![])
        };
        //查找数据库中配置角色
        //1. 游客,所有用户共享的角色
        //2. 指定关系角色,由外部传入的角色
        //3. 当user_id>0时,登录用户共享的角色,当前用户独有的角色
        macro_rules! find_role_data {
            ($find_obj:expr) => {
                if user_id > 0 {
                    let (relation, all_user, login, user) = tokio::try_join!(
                        $find_obj.find_role_by_relation(relation_key_roles, &res_vec),
                        $find_obj.find_role_by_all_user(&res_vec),
                        $find_obj.find_role_by_login_user(&res_vec),
                        $find_obj.find_role_by_user(user_id, &res_vec),
                    )?;
                    role_data = role_data.merge(relation);
                    role_data = role_data.merge(all_user);
                    role_data = role_data.merge(login);
                    role_data = role_data.merge(user);
                } else {
                    let (relation, all_user) = tokio::try_join!(
                        $find_obj.find_role_by_relation(relation_key_roles, &res_vec),
                        $find_obj.find_role_by_all_user(&res_vec),
                    )?;
                    role_data = role_data.merge(relation);
                    role_data = role_data.merge(all_user);
                }
            };
        }
        if self.use_cache {
            //使用权限缓存
            let cache = self.role.cache();
            find_role_data!(cache);
        } else {
            find_role_data!(self.role);
        }
        //对获取到所有角色跟待验证资源进行授权验证
        let mut bad_tmp = vec![];
        for check_item in res_vec.iter() {
            for res_op in check_item.ops.iter() {
                let mut access = (
                    false,
                    fluent_message!("rbac-access-check-access",{
                            "res":check_item.res.name,
                            "res_id":check_item.res.id,
                            "user_id":check_item.res.user_id,
                            "res_op":res_op.op_key,
                            "view_user_id":user_id
                    }), //"user[{$user_id}] not find access [{$res}:{$res_id}] on [{$res_op}]"
                );

                if let Some(role) = role_data.match_role(res_op.id) {
                    if role.is_pass() {
                        access.0 = true;
                        debug!(
                            "user {} access allow res op {} on role {}",
                            user_id,
                            res_op.id,
                            role.to_string()
                        );
                    } else {
                        info!(
                            "user {} access deny res op {} on role {}",
                            user_id,
                            res_op.id,
                            role.to_string()
                        );
                    }
                }
                if !access.0 {
                    info!(
                        "user {} access deny res :{},msg:{}",
                        user_id, check_item.res.name, access.1
                    );
                    bad_tmp.push((check_item.res.name.clone(), access.1));
                }
            }
        }
        if bad_tmp.is_empty() {
            Ok(())
        } else {
            //根据角色算出有未被授权资源
            Err(UserRbacError::Check(bad_tmp))
        }
    }
}
