use lsys_core::RequestEnv;
use lsys_rbac::dao::{
    OpInfo, RbacOpAddData, RbacOpData, RbacResAddData, RbacResData, ResInfo, ResTypeParam,
};

use super::WebRbac;
use crate::common::JsonResult;

pub struct RbacSyncResParam<'t> {
    pub res_type: &'t str,
    pub res_data: &'t str,
    pub init_res_name: Option<&'t str>,
}
//文章 -> 查看 ,编辑 =>{OP}
//文章 1
//查看,编辑
//类型:文章 操作:查看,编辑
impl WebRbac {
    //获取资源对应资源ID
    pub async fn sync_res_id<'t>(
        &self,
        user_id: u64,
        app_id: u64,
        res_key: &'t [RbacSyncResParam<'t>],
        init_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(&'t RbacSyncResParam<'t>, u64)>> {
        let keys = res_key
            .iter()
            .map(|e| ResInfo {
                res_type: e.res_type,
                res_data: e.res_data,
                user_id,
                app_id,
            })
            .collect::<Vec<_>>();
        let mut data = self
            .rbac_dao
            .res
            .find_vec_by_info(&keys.iter().collect::<Vec<_>>())
            .await?;
        let mut out = vec![];
        for key in res_key {
            let opt_id = if let Some(index) = data
                .iter()
                .position(|x| x.0.res_data == key.res_data && x.0.res_type == key.res_type)
            {
                data.remove(index).1.map(|e| e.id)
            } else {
                None
            };
            let model_id = match opt_id {
                Some(tmp) => tmp,
                None => {
                    self.rbac_dao
                        .res
                        .add_res(
                            &RbacResAddData {
                                user_id,
                                app_id: Some(app_id),
                                res_info: RbacResData {
                                    res_name: key.init_res_name,
                                    res_type: key.res_type,
                                    res_data: key.res_data,
                                },
                            },
                            init_user_id,
                            None,
                            env_data,
                        )
                        .await?
                }
            };
            out.push((key, model_id));
        }
        Ok(out)
    }
}

pub struct RbacSyncOpParam<'t> {
    pub op_key: &'t str,
    pub init_op_name: Option<&'t str>,
}

impl WebRbac {
    //同步指定资源跟操作关系并返回对应的操作ID
    pub async fn sync_res_type_op_id<'t>(
        &self,
        res_type_data: &ResTypeParam<'_>,
        op_key: &'t [&'t RbacSyncOpParam<'t>],
        init_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(&'t RbacSyncOpParam<'t>, u64)>> {
        let data = self
            .sync_op_id(
                res_type_data.user_id,
                res_type_data.app_id,
                op_key,
                init_user_id,
                env_data,
            )
            .await?;
        //关联资源类型+操作
        let op_res_data = self
            .rbac_dao
            .res
            .res_type_op_data(
                res_type_data,
                Some(&op_key.iter().map(|e| e.op_key).collect::<Vec<_>>()),
                false,
                None,
            )
            .await?;
        let op_vec_data = self
            .rbac_dao
            .op
            .find_by_ids(&data.iter().map(|e| e.1).collect::<Vec<_>>())
            .await?;

        let add_op_data = op_vec_data
            .into_iter()
            .filter(|(_, op_model)| !op_res_data.iter().any(|e| e.op_res.op_id == op_model.id))
            .map(|t| t.1)
            .collect::<Vec<_>>();
        self.rbac_dao
            .res
            .res_type_add_op(res_type_data, &add_op_data, init_user_id, None, env_data)
            .await?;
        Ok(data)
    }
    //获取操作对应的操作ID
    pub async fn sync_op_id<'t>(
        &self,
        user_id: u64,
        app_id: u64,
        op_key: &'t [&'t RbacSyncOpParam<'t>],
        init_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(&'t RbacSyncOpParam<'t>, u64)>> {
        let keys = op_key
            .iter()
            .map(|e| OpInfo {
                op_key: e.op_key,
                user_id,
                app_id,
            })
            .collect::<Vec<_>>();
        let mut data = self
            .rbac_dao
            .op
            .find_vec_by_info(&keys.iter().collect::<Vec<_>>())
            .await?;
        let mut out = vec![];
        for key in op_key {
            let opt_id = if let Some(index) = data.iter().position(|x| x.0.op_key == key.op_key) {
                data.remove(index).1.map(|e| e.id)
            } else {
                None
            };
            let model_id = match opt_id {
                Some(tmp) => tmp,
                None => {
                    self.rbac_dao
                        .op
                        .add_op(
                            &RbacOpAddData {
                                user_id,
                                app_id: Some(app_id),
                                op_info: RbacOpData {
                                    op_name: key.init_op_name,
                                    op_key: key.op_key,
                                },
                            },
                            init_user_id,
                            None,
                            env_data,
                        )
                        .await?
                }
            };
            out.push((*key, model_id));
        }
        Ok(out)
    }
}

pub struct RbacSyncResRecrod {
    pub res_type: String,
    pub res_data: String,
    pub res_id: u64,
    pub op_data: Vec<(String, u64)>,
}

impl WebRbac {
    //把资源及操作模板数据同步到数据库并返回结果
    pub async fn res_tpl_sync(
        &self,
        user_id: u64,
        app_id: u64,
        res_type: &str,
        res_data: &[impl AsRef<str>],
        init_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<RbacSyncResRecrod>> {
        let tpl_data = self.res_tpl_data(true, true);
        let res_id_str = res_data.iter().map(|e| e.as_ref()).collect::<Vec<_>>();
        let res_param = res_id_str
            .iter()
            .map(|e| RbacSyncResParam {
                res_type,
                res_data: e,
                init_res_name: None,
            })
            .collect::<Vec<_>>();
        let res_db_data = self
            .sync_res_id(user_id, app_id, &res_param, init_user_id, env_data)
            .await?;

        let op_data = if let Some(op_tpl) = tpl_data.iter().find(|e| e.key == res_type) {
            let key_data = op_tpl
                .ops
                .iter()
                .map(|e| RbacSyncOpParam {
                    op_key: e,
                    init_op_name: None,
                })
                .collect::<Vec<_>>();
            let key_data = key_data.iter().collect::<Vec<_>>();
            let tpl_data = self
                .sync_res_type_op_id(
                    &ResTypeParam {
                        res_type,
                        user_id,
                        app_id,
                    },
                    &key_data,
                    init_user_id,
                    env_data,
                )
                .await?;
            tpl_data
                .iter()
                .map(|(e, op_id)| (e.op_key.to_string(), *op_id))
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let mut out_data = vec![];

        for (res, res_id) in res_db_data {
            out_data.push(RbacSyncResRecrod {
                res_type: res.res_type.to_string(),
                res_data: res.res_data.to_string(),
                res_id,
                op_data: op_data.clone(),
            })
        }
        Ok(out_data)
    }
}
