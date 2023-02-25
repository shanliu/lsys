use std::{sync::Arc, vec};

use lsys_core::{get_message, FluentMessage};
use tracing::{debug, info};

use crate::model::{RbacResModel, RbacRoleModel, RbacRoleOpPositivity};

use super::{
    res::RbacRes,
    role::{RbacRole, RoleRelationKey},
    RbacResData, ResKey, UserRbacError, UserRbacResult, ROLE_PRIORITY_MAX, ROLE_PRIORITY_MIN,
    ROLE_PRIORITY_NONE,
};

// 授权前提
// 资源  属性: 1. 用户 2. 可执行操作【必须明确授权】
// 角色  访问用户对资源用户纬度: 1. 为指定关系 2.为特定分组关系 3.为任意用户

//需求
//用户 是否有对 资源【RbacResModel】 进行 资源某些操作的权限【RbacResOpModel】
//需明确:
//  资源 用户【全局，还是指定用户】

//核心流程
//用户 -> 角色 -> 资源(操作) -> 是否满足权限

//操作流程

//入参：
//  1. 进行访问用户id  @check的 user_id 参数
//  2. 需要访问资源及对该资源的操作,可能需要多个资源权限 @check的 check_vec 参数
//      1. 资源是系统还是某用户 【AccessRes.user_id】
//      2. 要资源的操作列表【【AccessRes.ops】 及这些操作的 默认是否需要授权【AccessResOp.must_authorize】
//      3. 涉及到多个权限时，存在任意一个未授权都认为未授权，即 check_vec 多个值
//  3. 资源所属用户 跟 进行访问用户 的角色关系key  @check的 relation_role 参数
//      1. 系统资源 关系key示例：会员等级
//      2. 特定用户资源 关系key示例：指定某些组或用户有查看权限
//  4. 访问某资源属于 访问用户 对该资源操作是否不检查权限 @check的 self_res_skip 参数

//  资源操作权限 默认是否需要授权：【AccessResOp.must_authorize】
//      1. 资源操作权限【RbacResModel】不存在资源记录即未创建
//          1. 默认需要授权【如后台页面：管理页面】，认为无权限
//          2. 默认不需要授权【如前台页面：登录，首页等】，认为有权限
//      2. 资源操作权限 存在记录 由角色管控
//  资源权限 分类：
//      1. 用户资源 user_id>0
//      2. 系统资源 user_id=0

//查询 进行访问用户 拥有角色
//  得到公共角色:
//      1. 系统资源 公共权限角色 即 【RbacRoleModel user_id=0】的角色
//      2. 如果为 被访问用户 的资源【资源user_id>0】,由被访问用户决定是否能被访问 进行访问用户
//         通过【传入自定义key:RbacRoleRelationModel + 该key的用户id，可以设置为 被访问用户ID】确定 被访问用户的关系
//        【传入自定义key】+  查 被访问用户 的对该关系用户 公共权限角色
//
//  得到 进行访问用户 被配置角色 通过 RbacRoleUserModel 获取

//合并 进行访问用户 拥有角色，根据 RbacRoleOpModel 得到 拥有的资源操作权限 和 禁止的资源操作权限
//在比较需要访问资源操作是否通过权限验证【存在任意一个禁止则无权操作】

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
                None => "model role:none".to_string(),
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

//需要验证但未被管理的资源
pub struct MustAuthorizeRes {
    pub res: Option<RbacResModel>,
    pub access_res: AccessRes, //资源用户ID
}
pub trait SystemRoleCheckData: Sync + Send {
    /// 过滤需要验证但未被未管理资源记录
    /// 返回为认为授权失败的未管理资源记录
    fn filter_must_authorize_res(
        &self,
        user_id: u64,
        items: Vec<MustAuthorizeRes>,
    ) -> Vec<MustAuthorizeRes>;
    /// 根据需要验证资源返回角色检测数据，跟数据库中查找到记录合并后进行权限校验
    fn role_check_data(&self, user_id: u64, check_vec: &[RbacResData]) -> RoleCheckData;
}

pub struct SystemRole {
    root_user_id: Vec<u64>,
    self_res: bool,
}
impl SystemRole {
    pub fn new(self_res: bool, root_user_id: Vec<u64>) -> Self {
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
    fn filter_must_authorize_res(
        &self,
        user_id: u64,
        items: Vec<MustAuthorizeRes>,
    ) -> Vec<MustAuthorizeRes> {
        if self.root_user_id.iter().any(|e| *e == user_id) {
            return vec![];
        }
        if self.self_res {
            return items
                .into_iter()
                .filter(|e| e.access_res.user_id != user_id)
                .collect::<Vec<MustAuthorizeRes>>();
        };
        items
    }
}

/// RBAC对外结构
pub struct RbacAccess {
    fluent: Arc<FluentMessage>,
    res: Arc<RbacRes>,
    role: Arc<RbacRole>,
    system_role: Option<Box<dyn SystemRoleCheckData>>,
    use_cache: bool,
}

//待检查授权资源结构
#[derive(Clone, Debug)]
pub struct AccessRes {
    pub res: String,           //资源KEY
    pub user_id: u64,          //资源用户ID
    pub ops: Vec<AccessResOp>, //授权列表
}

//资源授权结构
#[derive(Clone, Debug)]
pub struct AccessResOp {
    pub op: String,           //授权KEY
    pub must_authorize: bool, //是否配置授权才能访问
}

//检测菜单结构
#[derive(Clone, Debug)]
pub struct MenuAccess {
    pub access_res: Vec<Vec<AccessRes>>, //该菜单涉及权限列表
    pub name: String,                    //菜单名或KEY
}

//菜单授权检查结果
pub struct MenuResult {
    pub result: UserRbacResult<()>, //是否授权成功
    pub name: String,               //菜单名或key,参见:MenuAccess.name
}

impl RbacAccess {
    pub fn new(
        fluent: Arc<FluentMessage>,
        res: Arc<RbacRes>,
        role: Arc<RbacRole>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        use_cache: bool,
    ) -> Self {
        RbacAccess {
            fluent,
            res,
            role,
            system_role,
            use_cache,
        }
    }
    /// 批量权限检测
    /// 使用场景:
    ///     一些菜单上每个菜单都需要一定的授权才能被访问
    ///     这个方法传入菜单及关联需要权限数据
    ///     返回每个菜单上的关联的授权检查是否通过
    pub async fn menu_check(
        &self,
        user_id: u64,
        relation_role: &[RoleRelationKey],
        menu_vec: &[MenuAccess],
    ) -> Vec<MenuResult> {
        let mut out = Vec::with_capacity(menu_vec.len());
        for e in menu_vec.iter() {
            out.push(MenuResult {
                name: e.name.to_owned(),
                result: self.check(user_id, relation_role, &e.access_res).await,
            })
        }
        out
    }
    /// 合并待检查授权资源结构数据
    /// 使用场景示例:
    ///     一个发送短信功能需要一些权限
    ///     一个发送邮件的功能也需要一些权限
    ///     现在`操作的功能`需要发送短信跟需要发送邮件，所以必须把上面需要的权限做合并
    ///     可通过该方法来合并权限数据在验证该`操作的功能`是被授权
    pub fn merge_access_res(
        res1: &[Vec<AccessRes>],
        res2: &[Vec<AccessRes>],
    ) -> Vec<Vec<AccessRes>> {
        //外层卡迪尔积，内层合并需检查授权
        let mut res = Vec::with_capacity(res1.len() * res2.len());
        for tmp1 in res1 {
            for tmp2 in res2 {
                let mut tt = Vec::with_capacity(tmp1.len());
                for r1 in tmp1 {
                    let mut tt1 = r1.ops.clone();
                    for r2 in tmp2 {
                        if r1.res == r2.res && r1.user_id == r2.user_id {
                            for o1 in r2.ops.iter() {
                                let mut find = false;
                                for ot1 in tt1.iter_mut() {
                                    if ot1.op == o1.op {
                                        find = true;
                                        ot1.must_authorize =
                                            o1.must_authorize || ot1.must_authorize;
                                        break;
                                    }
                                }
                                if !find {
                                    tt1.push(o1.to_owned())
                                }
                            }
                        }
                    }
                    tt.push(AccessRes {
                        res: r1.res.to_owned(),
                        user_id: r1.user_id,
                        ops: tt1,
                    })
                }
                for r2 in tmp2 {
                    if !tmp1
                        .iter()
                        .any(|r1| r1.res == r2.res && r1.user_id == r2.user_id)
                    {
                        tt.push(r2.to_owned())
                    }
                }
                res.push(tt);
            }
        }
        res
    }
    /// 指定权限检测
    /// 使用场景: 访问系统功能时验证是否有访问权限
    pub async fn check(
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
        if check_vec.is_empty() {
            return Ok(());
        }
        let mut bad_data = vec![];
        for check_res in check_vec {
            let res_list = check_res
                .iter()
                .map(|e| ResKey {
                    res_key: e.res.clone(),
                    user_id: e.user_id,
                })
                .collect::<Vec<ResKey>>();
            if res_list.is_empty() {
                return Ok(());
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
                        .filter(|e| acc_res.ops.iter().any(|ee| ee.op == e.op_key))
                        .collect::<Vec<_>>();
                } else {
                    res_data.ops = vec![];
                }
                res_data
            })
            .collect::<Vec<_>>();
            //需要验证且不存在数据库记录的资源
            let mut not_find_res_vec = vec![];
            for acc_res in check_res.iter() {
                let mut find_res = false;
                for res in res_vec.iter() {
                    //res 已存在在数据库的资源
                    if res.res.res_key == acc_res.res {
                        find_res = true;
                        let mut bad_op_vec = vec![];
                        //acc_res 需要检测的资源
                        for oe in acc_res.ops.iter() {
                            if !res
                                .ops
                                .iter()
                                .any(|t| t.op_key == oe.op && oe.must_authorize)
                            {
                                bad_op_vec.push(oe.to_owned());
                            }
                        }
                        not_find_res_vec.push(MustAuthorizeRes {
                            res: Some(res.res.to_owned()),
                            access_res: AccessRes {
                                res: acc_res.res.clone(),
                                user_id: acc_res.user_id,
                                ops: bad_op_vec,
                            },
                        });
                        break;
                    }
                }
                if !find_res {
                    not_find_res_vec.push(MustAuthorizeRes {
                        res: None,
                        access_res: acc_res.to_owned(),
                    });
                }
            }
            if let Some(ref sys_role) = self.system_role {
                //系统角色过滤需要验证但不存在数据库记录的资源
                not_find_res_vec = sys_role.filter_must_authorize_res(user_id, not_find_res_vec);
            };

            let mut bad_tmp = vec![];
            let res_vec = res_vec
                .into_iter()
                .filter(|e| !e.ops.is_empty())
                .collect::<Vec<RbacResData>>();
            if res_vec.is_empty() {
                //不存在需要验证的资源
                if not_find_res_vec.is_empty() {
                    //不存在 需要验证且不存在数据库记录的资源
                    return Ok(());
                } else {
                    //存在 需要验证且不存在数据库记录的资源
                    //记录验证失败的记录，跳出当前循环
                    for tmp_res in not_find_res_vec {
                        match tmp_res.res {
                            Some(res) => {
                                for tmp_op in tmp_res.access_res.ops {
                                    bad_tmp.push((tmp_res.access_res.res.clone(),
                                        get_message!(&self.fluent, "rbac-access-check-res-empty", "Authorization not find user [{$user_id}] in res[{$res}:{$res_id}] on op {$op} [{$view_user}]",[
                                        "res_id"=>res.id,
                                        "res"=>tmp_res.access_res.res.clone(),
                                        "op"=>tmp_op.op,
                                        "user_id"=>tmp_res.access_res.user_id,
                                        "view_user"=>user_id
                                    ])));
                                }
                            }
                            None => {
                                for tmp_op in tmp_res.access_res.ops {
                                    bad_tmp.push((tmp_res.access_res.res.clone(),
                                        get_message!(&self.fluent, "rbac-access-check-res-empty", "Authorization not find  user [{$user_id}] res [{$res}] on op {$op} [{$view_user}]",[
                                        "res"=>tmp_res.access_res.res.clone(),
                                        "op"=>tmp_op.op,
                                        "user_id"=>tmp_res.access_res.user_id,
                                        "view_user"=>user_id
                                    ])));
                                }
                            }
                        }
                    }
                    bad_data.append(&mut bad_tmp);
                    continue;
                }
            }
            //获取系统角色
            let mut role_data = if let Some(ref sys_role) = self.system_role {
                sys_role.role_check_data(user_id, &res_vec)
            } else {
                RoleCheckData::new(vec![])
            };
            macro_rules! find_role_data {
                ($find_obj:expr) => {
                    if user_id > 0 {
                        let (relation, guest, login, user) = tokio::try_join!(
                            $find_obj.find_role_by_relation(relation_key_roles, &res_vec),
                            $find_obj.find_role_by_guest_user(&res_vec),
                            $find_obj.find_role_by_login_user(&res_vec),
                            $find_obj.find_role_by_user(user_id, &res_vec),
                        )?;
                        role_data = role_data.merge(relation);
                        role_data = role_data.merge(guest);
                        role_data = role_data.merge(login);
                        role_data = role_data.merge(user);
                    } else {
                        let (relation, guest) = tokio::try_join!(
                            $find_obj.find_role_by_relation(relation_key_roles, &res_vec),
                            $find_obj.find_role_by_guest_user(&res_vec),
                        )?;
                        role_data = role_data.merge(relation);
                        role_data = role_data.merge(guest);
                    }
                };
            }
            if self.use_cache {
                let cache = self.role.cache();
                find_role_data!(cache);
            } else {
                find_role_data!(self.role);
            }

            for check_item in res_vec.iter() {
                for res_op in check_item.ops.iter() {
                    let mut access = (
                        false,
                        get_message!(&self.fluent, "rbac-access-check-access", "user[{$user_id}] not find access [{$res}:{$res_id}] on [{$res_op}]",[
                            "res"=>check_item.res.name.clone(),
                            "res_id"=>check_item.res.id,
                            "res_op"=>res_op.op_key.clone(),
                            "user_id"=>user_id
                        ]),
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
                return Ok(());
            } else {
                bad_data.append(&mut bad_tmp);
            }
        }
        Err(UserRbacError::Check(bad_data))
    }
}
