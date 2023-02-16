use std::vec;

use lsys_rbac::model::RbacRoleResOpRange;

use crate::{JsonData, JsonResult};

macro_rules! res_data {
    ($res:ident) => {
        crate::dao::access::ResData::$res.to_access_res()?
    };
    ($res:ident($($var:expr),+$(,)*)) => {
        crate::dao::access::ResData::$res($($var),+).to_access_res()?
    };
}

pub struct RoleOpCheck {
    pub op_id: u64,
    pub op_user_id: u64,
}

pub enum ResData {
    //admin
    AdminManage,
    AdminSetting,
    AdminAliSmsConfig,
    AdminTest,
    //system res
    SystemReSetPassword,
    SystemLogin,
    SystemEmailConfirm,
    SystemMobileConfirm,
    //user res
    UserEmailView(u64),
    UserEmailEdit(u64),
    UserMobileView(u64),
    UserMobileEdit(u64),
    UserAddressView(u64),
    UserAddressEdit(u64),
    UserExternalEdit(u64),
    UserSetPassword(u64),
    UserNameEdit(u64),
    UserInfoEdit(u64),
    UserResView(u64),
    UserResEdit(u64),
    UserRoleView(u64),
    UserRoleViewList(Vec<u64>),
    UserRoleAdd(u64, i8, Option<Vec<RoleOpCheck>>),
    UserRoleEdit(u64, Option<i8>, Option<Vec<RoleOpCheck>>),
    UserResAllView(u64),
    //user app
    UserViewApp(u64),
    UserEditApp(u64),
    UserConfirmApp,
    //app
    AppView(u64, u64),
    AppSender(u64, u64),
    AppRbacCheck(u64),
}

macro_rules! user_res {
    ($user_id:expr,$access_res:expr) => {
        if *$user_id == 0 {
            Err(JsonData::message_error("user id is wrong"))
        } else {
            Ok($access_res)
        }
    };
}

impl ResData {
    /// 转换为待检测资源
    #[allow(clippy::type_complexity)]
    pub fn to_access_data(
        &self,
    ) -> JsonResult<(Vec<Vec<lsys_rbac::dao::AccessRes>>, Option<Vec<Self>>)> {
        match self {
            ResData::AdminManage => Ok((
                vec![access_res!(["admin", access_op!(["view", true])])],
                None,
            )),
            ResData::AdminTest => Ok((
                vec![access_res!(["admin", access_op!(["test", true])])],
                Some(vec![Self::AdminManage]),
            )),
            ResData::AdminSetting => Ok((
                vec![access_res!(["admin", access_op!(["setting", true])])],
                Some(vec![Self::AdminManage]),
            )),
            ResData::AdminAliSmsConfig => Ok((
                vec![access_res!(["admin", access_op!(["alisms-config", true])])],
                Some(vec![Self::AdminManage]),
            )),
            ResData::AppView(app_id, user_id) => Ok((
                vec![
                    access_res!([
                        format!("app-{}", app_id),
                        access_op!(["global-app-view", true])
                    ]),
                    access_res!([
                        format!("app-{}", app_id),
                        *user_id,
                        access_op!(["app-view", true])
                    ]),
                ],
                None,
            )),
            ResData::AppSender(app_id, user_id) => Ok((
                vec![
                    access_res!([
                        format!("app-{}", app_id),
                        access_op!(["global-app-sender-config", true])
                    ]),
                    access_res!([
                        format!("app-{}", app_id),
                        *user_id,
                        access_op!(["app-sender-config", true])
                    ]),
                ],
                None,
            )),
            ResData::AppRbacCheck(app_id) => Ok((
                vec![access_res!([
                    format!("app-{}", app_id),
                    access_op!(["global-rbac-check", true])
                ])],
                None,
            )),
            ResData::UserConfirmApp => Ok((
                vec![access_res!([
                    "app",
                    access_op!(["global-app-confirm", true])
                ])],
                Some(vec![Self::AdminManage]),
            )),
            ResData::SystemLogin => Ok((
                vec![access_res!(["user", access_op!(["global-login", false])])],
                None,
            )),
            ResData::SystemEmailConfirm => Ok((
                vec![access_res!([
                    "user",
                    access_op!(["global-email-confirm", false])
                ])],
                None,
            )),

            ResData::SystemMobileConfirm => Ok((
                vec![access_res!([
                    "user",
                    access_op!(["global-mobile-confirm", false])
                ])],
                None,
            )),
            ResData::SystemReSetPassword => Ok((
                vec![access_res!([
                    "user",
                    access_op!(["global-reset-password", false])
                ])],
                None,
            )),
            ResData::UserAddressView(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["address-view", true])]),
                        access_res!(["user", access_op!(["global-address-view", true])]),
                    ],
                    None,
                )
            ),
            ResData::UserAddressEdit(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["address-edit", true])]),
                        access_res!(["user", access_op!(["global-address-edit", true])]),
                    ],
                    Some(vec![Self::UserAddressView(*user_id)]),
                )
            ),
            ResData::UserExternalEdit(user_id) => user_res!(
                user_id,
                (
                    vec![access_res!([
                        "user",
                        *user_id,
                        access_op!(["external-change", true])
                    ])],
                    None,
                )
            ),
            ResData::UserSetPassword(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["set-password", true])]),
                        access_res!(["user", access_op!(["global-set-password", true])]),
                    ],
                    None,
                )
            ),

            ResData::UserNameEdit(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["name-edit", true])]),
                        access_res!(["user", access_op!(["global-name-edit", true])]),
                    ],
                    None,
                )
            ),
            ResData::UserInfoEdit(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["info-edit", true])]),
                        access_res!(["user", access_op!(["global-info-edit", true])]),
                    ],
                    None,
                )
            ),
            ResData::UserEmailView(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["email-view", true])]),
                        access_res!(["user", access_op!(["global-email-view", true])]),
                    ],
                    None,
                )
            ),
            ResData::UserEmailEdit(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["email-edit", true])]),
                        access_res!(["user", access_op!(["global-email-edit", true])]),
                    ],
                    Some(vec![Self::UserEmailView(*user_id)]),
                )
            ),
            ResData::UserViewApp(user_id) => {
                if *user_id == 0 {
                    Ok((
                        vec![access_res!(["app", access_op!(["global-app-view", true])])],
                        None,
                    ))
                } else {
                    Ok((
                        vec![access_res!([
                            "app",
                            *user_id,
                            access_op!(["app-view", true])
                        ])],
                        None,
                    ))
                }
            }
            ResData::UserEditApp(user_id) => {
                if *user_id == 0 {
                    Ok((
                        vec![access_res!(["app", access_op!(["global-app-edit", true])])],
                        None,
                    ))
                } else {
                    Ok((
                        vec![access_res!([
                            "app",
                            *user_id,
                            access_op!(["app-edit", true])
                        ])],
                        None,
                    ))
                }
            }
            ResData::UserMobileView(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["mobile-view", true])]),
                        access_res!(["user", access_op!(["global-mobile-view", true])]),
                    ],
                    None,
                )
            ),
            ResData::UserMobileEdit(user_id) => user_res!(
                user_id,
                (
                    vec![
                        access_res!(["user", *user_id, access_op!(["mobile-edit", true])]),
                        access_res!(["user", access_op!(["global-mobile-edit", true])]),
                    ],
                    Some(vec![Self::UserMobileView(*user_id)]),
                )
            ),
            ResData::UserResEdit(_user_id) => Ok((
                vec![
                    //自己不能管理自己资源,通过系统权限管理
                    // access_res!(["rbac", *user_id, access_op!(["res-change", true])]),
                    access_res!(
                        ["admin", access_op!(["view", true])],
                        [
                            "rbac",
                            access_op!(["global-res-view", true], ["global-res-change", true])
                        ]
                    ),
                ],
                None,
            )),
            ResData::UserResView(user_id) => Ok((
                vec![
                    access_res!(["rbac", *user_id, access_op!(["res-view", true])]),
                    access_res!(["rbac", access_op!(["global-res-view", true])]),
                ],
                None,
            )),
            ResData::UserRoleView(user_id) => Ok((
                vec![
                    access_res!(["rbac", *user_id, access_op!(["role-view", true])]),
                    access_res!(["rbac", access_op!(["global-role-view", true])]),
                ],
                None,
            )),
            ResData::UserRoleAdd(user_id, op_range, op_param) => {
                let mut gres = access_res!(["rbac", *user_id, access_op!(["role-change", true])]);
                if RbacRoleResOpRange::AllowAll.eq(*op_range) {
                    gres.push(access_res!("rbac", access_op!(["role-allow-res", true])))
                } else if RbacRoleResOpRange::DenyAll.eq(*op_range) {
                    gres.push(access_res!("rbac", access_op!(["role-deny-res", true])))
                }
                if let Some(rop) = op_param {
                    for tmp in rop {
                        if *user_id > 0 && tmp.op_user_id != *user_id {
                            return Err(JsonData::message_error(
                                "can't add other user res to your role",
                            ));
                        }
                    }
                }
                Ok((
                    vec![
                        gres,
                        access_res!(["rbac", *user_id, access_op!(["global-role-change", true])]),
                    ],
                    Some(vec![Self::UserResView(*user_id)]),
                ))
            }
            ResData::UserRoleEdit(user_id, op_range_opt, op_param) => {
                let mut gres = access_res!(["rbac", *user_id, access_op!(["role-change", true])]);
                if let Some(op_range) = op_range_opt {
                    if RbacRoleResOpRange::AllowAll.eq(*op_range) {
                        gres.push(access_res!("rbac", access_op!(["role-allow-res", true])));
                    } else if RbacRoleResOpRange::DenyAll.eq(*op_range) {
                        gres.push(access_res!("rbac", access_op!(["role-deny-res", true])));
                    }
                };
                if let Some(rop) = op_param {
                    for tmp in rop {
                        if *user_id > 0 && tmp.op_user_id != *user_id {
                            return Err(JsonData::message_error(
                                "can't add other user res to your role",
                            ));
                        }
                    }
                }
                Ok((
                    vec![
                        gres,
                        access_res!(["rbac", *user_id, access_op!(["global-role-change", true])]),
                    ],
                    Some(vec![Self::UserResView(*user_id)]),
                ))
            }
            ResData::UserRoleViewList(user_ids) => {
                let user_ids = user_ids.to_vec();
                if user_ids.is_empty() {
                    Ok((vec![], None))
                } else {
                    let roles = user_ids
                        .into_iter()
                        .collect::<std::collections::HashSet<u64>>()
                        .iter()
                        .map(|e| access_res!("rbac", *e, access_op!(["role-view", true])))
                        .collect::<Vec<lsys_rbac::dao::AccessRes>>();
                    Ok((
                        vec![
                            roles,
                            access_res!(["rbac", access_op!(["global-role-view", true])]),
                        ],
                        None,
                    ))
                }
            }
            ResData::UserResAllView(user_id) => Ok((
                vec![
                    access_res!(["rbac", *user_id, access_op!(["res-view-all", true])]),
                    access_res!(["rbac", access_op!(["global-res-view-all", true])]),
                ],
                None,
            )),
        }
    }
    pub fn global_res_data() -> Vec<ResData> {
        vec![
            Self::SystemReSetPassword,
            Self::SystemLogin,
            Self::SystemEmailConfirm,
            Self::SystemMobileConfirm,
        ]
    }
    pub fn user_res_data(user_id: u64) -> Vec<ResData> {
        vec![
            Self::UserEmailView(user_id),
            Self::UserEmailEdit(user_id),
            Self::UserMobileView(user_id),
            Self::UserMobileEdit(user_id),
            Self::UserAddressView(user_id),
            Self::UserAddressEdit(user_id),
            Self::UserExternalEdit(user_id),
            Self::UserSetPassword(user_id),
            Self::UserNameEdit(user_id),
            Self::UserInfoEdit(user_id),
            Self::UserResView(user_id),
            Self::UserResEdit(user_id),
            Self::UserRoleView(user_id),
            Self::UserRoleEdit(user_id, None, None),
        ]
    }
    /// 系统资源列表
    pub fn all_res(user_id: u64) -> Vec<lsys_rbac::dao::AccessRes> {
        let mut list_res = if user_id == 0 {
            Self::global_res_data()
        } else {
            vec![]
        };
        list_res.extend(Self::user_res_data(if user_id == 0 { 1 } else { user_id }));
        Self::merge_access_res(&list_res, user_id).unwrap_or_default()
    }
    /// 系统资源列表
    pub fn user_res(user_id: u64) -> Vec<lsys_rbac::dao::AccessRes> {
        Self::merge_access_res(
            &[Self::UserResView(user_id), Self::UserResEdit(user_id)],
            user_id,
        )
        .unwrap_or_default()
    }
}
impl ResData {
    /// 转换合并ResData为资源列表
    fn merge_access_res(
        res_list: &[ResData],
        filter_user_id: u64,
    ) -> JsonResult<Vec<lsys_rbac::dao::AccessRes>> {
        let mut vout = Vec::<lsys_rbac::dao::AccessRes>::with_capacity(res_list.len());
        for tmp in res_list {
            let res_data = tmp.to_access_data()?;
            for res in res_data.0 {
                for rtmp in res {
                    let mut find = false;
                    for vtmp in vout.iter_mut() {
                        if vtmp.res == rtmp.res && rtmp.user_id == filter_user_id {
                            find = true;
                            for ott in vtmp.ops.iter_mut() {
                                for top in rtmp.ops.iter() {
                                    if ott.op == top.op {
                                        ott.must_authorize =
                                            ott.must_authorize || top.must_authorize;
                                        break;
                                    }
                                }
                            }
                            for top in rtmp.ops.iter() {
                                if !vtmp.ops.iter().any(|e| e.op == top.op) {
                                    vtmp.ops.push(top.to_owned())
                                }
                            }
                        }
                    }
                    if !find && filter_user_id == rtmp.user_id {
                        vout.push(rtmp);
                    }
                }
            }
        }
        Ok(vout)
    }
    pub fn to_access_res(&self) -> JsonResult<Vec<Vec<lsys_rbac::dao::AccessRes>>> {
        let (mut res, dependency) = self.to_access_data()?;
        if let Some(dep) = dependency {
            for tmp in dep {
                res = lsys_rbac::dao::RbacAccess::merge_access_res(&tmp.to_access_res()?, &res);
            }
        }
        Ok(res)
    }
}
