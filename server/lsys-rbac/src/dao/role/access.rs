
use lsys_core::now_time;
//RBAC中角色相关实现
use sqlx::{FromRow, Row};
use lsys_core::sql_format;
use lsys_core::db::ModelTableName;
use tracing::error;

use crate::{dao::result::RbacResult, 
    model::{
        RbacOpModel, RbacPermModel, RbacPermStatus, RbacResModel, RbacRoleModel, RbacRoleResRange, 
        RbacRoleStatus, RbacRoleUserModel, RbacRoleUserRange, RbacRoleUserStatus
    }};

use super::{cache::RbacRoleCache, RbacRole};
use lsys_core::db::SqlQuote;

//角色对应授权检查的相关实现

//需检查权资源
#[derive(Clone, Debug)]
pub struct AccessResInfo<'t>{
    pub user_id:u64,
    pub app_id:u64,
    pub res_data:Vec<(&'t RbacResModel, Vec<&'t RbacOpModel>)>,
}
#[derive(Clone, Debug)]
pub struct AccessRoleInfo<'t>{
    pub app_id:u64,
    pub user_id:u64,
    pub role_key:&'t str,
}

impl RbacRoleCache<'_>{
    pub(crate) async fn clear_access(
        &self,
        role:&RbacRoleModel,//修改前model
        role_prem:Option<&[(u64,u64)]>,//res id,op id 所有变动
        role_user:Option<&[u64]>,//user id 所有变动
    ){
        //user_id res_range user_range 添加后不可修改
        if RbacRoleUserRange::Session.eq(role.user_range){
            if RbacRoleResRange::Any.eq(role.res_range){
                let key=self.role.cache().find_access_key_session_res_all(role.user_id, role.app_id,&role.role_key);
                self.role.cache_access.clear(&key).await;
            }else if RbacRoleResRange::Exclude.eq(role.res_range) ||  RbacRoleResRange::Include.eq(role.res_range){
                match role_prem{
                    Some(prem) => {
                        for (res_id,op_id) in prem {
                            let key= self.role.cache().find_access_key_session_by_res(role.user_id, role.app_id,&role.role_key,role.res_range,*res_id,*op_id);
                            self.role.cache_access.clear(&key).await;
                        }
                    },
                    None => {
                        let mut start_id=0;
                        loop {
                            match sqlx::query_as::<_, (u64, u64,u64)>(&sql_format!(
                                "select id,res_id,op_id from {} where role_id={} and id>{} order by id asc limit 100 ",
                                RbacPermModel::table_name(),
                                role.id,
                                start_id
                            ))
                            .fetch_all(&self.role.db)
                            .await{
                                Ok(data) => {
                                    if data.is_empty() {
                                        break;
                                    }
                                    for (id,res_id,op_id) in data {
                                        start_id=id;
                                        let key= self.role.cache().find_access_key_session_by_res(role.user_id, role.app_id,&role.role_key,role.res_range,res_id,op_id);
                                        self.role.cache_access.clear(&key).await;
                                    }
                                },
                                Err(err) => {
                                    //err
                                    error!("clear rbac cache fail on session:{err}");
                                    break;
                                },
                            }
                        }
                    },
                }
            }
        }else if RbacRoleUserRange::Custom.eq(role.user_range){
            if RbacRoleResRange::Any.eq(role.res_range){
                //自定义用户 -> 任意资源
                match role_user{
                    Some(role_user_data) => {
                        for access_user_id in role_user_data {                            
                            let key=self.role.cache().find_access_key_user_res_all(role.user_id, role.app_id,*access_user_id);
                            self.role.cache_access.clear(&key).await;
                        }
                    },
                    None => {
                        let mut start_id=0;
                        loop {
                            match sqlx::query_as::<_, (u64, u64)>(&sql_format!(
                                "select id,user_id from {} where role_id={} and id>{} order by id asc limit 100 ",
                                RbacRoleUserModel::table_name(),
                                role.id,
                                start_id
                            ))
                            .fetch_all(&self.role.db)
                            .await{
                                Ok(data) => {
                                    if data.is_empty() {
                                        break;
                                    }
                                    for (id,access_user_id) in data {
                                        start_id=id;
                                        let key=self.role.cache().find_access_key_user_res_all(role.user_id, role.app_id,access_user_id);
                                        self.role.cache_access.clear(&key).await;
                                    }
                                },
                                Err(err) => {
                                    //err
                                    error!("clear rbac cache fail on user all:{err}");
                                    break;
                                },
                            }
                        }
                    },
                }
            }else if RbacRoleResRange::Exclude.eq(role.res_range) ||  RbacRoleResRange::Include.eq(role.res_range){
                match role_prem{
                    Some(prem) => {
                        for (res_id,op_id) in prem {
                          
                            match role_user{
                                Some(role_user_data) => {
                                    for access_user_id in role_user_data {                            
                                        let key=self.role.cache().find_access_key_user_by_res(role.user_id, role.app_id,*access_user_id,role.res_range,*res_id,*op_id);
                                        self.role.cache_access.clear(&key).await;
                                    }
                                },
                                None => {
                                    let mut start_id=0;
                                    loop {
                                        match sqlx::query_as::<_, (u64, u64)>(&sql_format!(
                                            "select id,user_id from {} where role_id={} and id>{} order by id asc limit 100 ",
                                            RbacRoleUserModel::table_name(),
                                            role.id,
                                            start_id
                                        ))
                                        .fetch_all(&self.role.db)
                                        .await{
                                            Ok(data) => {
                                                if data.is_empty() {
                                                    break;
                                                }
                                                for (id,access_user_id) in data {
                                                    start_id=id;
                                                    let key=self.role.cache().find_access_key_user_by_res(role.user_id, role.app_id,access_user_id,role.res_range,*res_id,*op_id);
                                                    self.role.cache_access.clear(&key).await;
                                                }
                                            },
                                            Err(err) => {
                                                //err
                                                error!("clear rbac cache fail on user empty user:{err}");
                                                break;
                                            },
                                        }
                                    }
                                },
                            }
                        }
                    },
                    None => {
                        let mut start_id=0;
                        loop {
                            match sqlx::query_as::<_, (u64, u64,u64)>(&sql_format!(
                                "select id,res_id,op_id from {}  where role_id={} and id>{} order by id asc limit 100 ",
                                RbacPermModel::table_name(),
                                role.id,
                                start_id
                            ))
                            .fetch_all(&self.role.db)
                            .await{
                                Ok(data) => {
                                    if data.is_empty() {
                                        break;
                                    }
                                    for (id,res_id,op_id) in data {
                                        start_id=id;

                                        match role_user{
                                            Some(role_user_data) => {
                                                for access_user_id in role_user_data {                            
                                                    let key=self.role.cache().find_access_key_user_by_res(role.user_id, role.app_id,*access_user_id,role.res_range,res_id,op_id);
                                                    self.role.cache_access.clear(&key).await;
                                                }
                                            },
                                            None => {
                                                let mut user_start_id=0;
                                                loop {
                                                    match sqlx::query_as::<_, (u64, u64)>(&sql_format!(
                                                        "select id,user_id from {} where role_id={} and id>{} order by id asc limit 100 ",
                                                        RbacRoleUserModel::table_name(),
                                                        role.id,
                                                        user_start_id
                                                    ))
                                                    .fetch_all(&self.role.db)
                                                    .await{
                                                        Ok(data) => {
                                                            if data.is_empty() {
                                                                break;
                                                            }
                                                            for (id,access_user_id) in data {
                                                                user_start_id=id;
                                                                let key=self.role.cache().find_access_key_user_by_res(role.user_id,role.app_id, access_user_id,role.res_range,res_id,op_id);
                                                                self.role.cache_access.clear(&key).await;
                                                            }
                                                        },
                                                        Err(err) => {
                                                            //err
                                                            error!("clear rbac cache fail on user empty any:{err}");
                                                            break;
                                                        },
                                                    }
                                                }
                                            },
                                        }
                                    }
                                },
                                Err(err) => {
                                    //err
                                    error!("clear rbac cache fail on user empty prem:{err}");
                                    break;
                                },
                            }
                        }
                    },
                }
            }
        }
    }
    fn find_access_key_session_res_all(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        session_role_key:&str,
    ) ->String{
        format!("ur:{}-rr:{}-uid:{}-appid:{}-rkey:{}",RbacRoleUserRange::Session as i8,RbacRoleResRange::Any as i8,role_user_id,role_user_app_id,session_role_key)
    }
    fn find_access_key_session_by_res(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        session_role_key:&str,
        res_range:i8,
        res_id:u64,
        op_id:u64,
    ) ->String{   
        format!("ur:{}-rr:{}-uid:{}-appid:{}-rkey:{}-rid:{}-opid:{}",RbacRoleUserRange::Session as i8,res_range ,role_user_id,role_user_app_id,session_role_key,res_id,op_id)
    }
    fn find_access_key_user_res_all(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        access_user_id:u64,
    ) ->String{   
        format!("ur:{}-rr:{}-uid:{}-appid:{}-auid:{}",RbacRoleUserRange::Custom  as i8,RbacRoleResRange::Any as i8,role_user_id,role_user_app_id,access_user_id)
    }
    fn find_access_key_user_by_res(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        access_user_id:u64,
        res_range:i8,
        res_id:u64,
        op_id:u64,
    ) ->String{
        format!("ur:{}-rr:{}-uid:{}-appid:{}-auid:{}-rid:{}-opid:{}",RbacRoleUserRange::Custom as i8,res_range ,role_user_id,role_user_app_id,access_user_id,res_id,op_id)
    }
    pub async fn find_access_row<'s>(
        &self,
        //0 为游客 或具体的访问用户id
        user_id: u64,
        //待检测资源需要操作的列表
        check_res_data: &'s [AccessResInfo<'s>],
        check_role_data:&'s [AccessRoleInfo<'s>],
    )->RbacResult<AccessRoleData>{
        let mut role_find= RbacRoleFinder::new(self.role);
        //系统全局
        for role_data in check_role_data{
            if role_data.user_id>0{continue;}
             //系统-会话
            role_find.find_access_session_res_all(0,0,role_data.role_key).await;
           
        }
        if user_id>0{
            //系统-特定用户
            role_find.find_access_user_res_all(0, 0,user_id).await;
        }
        for res_data in check_res_data{
            //用户-公共
            if res_data.user_id>0{
                //用户全局
                for role_data in check_role_data{
                    if role_data.user_id!=res_data.user_id{continue;}
                    //用户-会话
                    role_find.find_access_session_res_all(res_data.user_id,role_data.app_id,role_data.role_key).await;
                }
                if user_id>0{
                    //用户-特定用户
                    role_find.find_access_user_res_all(res_data.user_id,res_data.app_id, user_id).await;
                }
            }
            for (res_item,op_data) in res_data.res_data.iter(){
                //跟资源相关
                for op_item in op_data.iter(){
                    //系统-排除或指定
                    // //系统-特定资源
                    for role_data in check_role_data{
                        if role_data.user_id>0{continue;}
                        //系统-会话
                        role_find.find_access_session_by_res(
                            0, 
                            0,
                            role_data.role_key,
                            res_item.id,
                            op_item.id,
                        ).await;
                    }
                    if user_id>0{
                        //系统-特定用户
                        role_find.find_access_user_by_res(
                            0, 
                            0,
                            user_id,
                            res_item.id,
                            op_item.id,
                        ).await;
                    }
                    //用户 - 排除或指定
                    if res_data.user_id>0{
                        for role_data in check_role_data{
                            if role_data.user_id!=res_data.user_id{continue;}
                            //用户-会话
                            role_find.find_access_session_by_res(
                                res_data.user_id,
                                role_data.app_id,
                                role_data.role_key,
                                res_item.id,
                                op_item.id,
                            ).await;
                        }
                        if user_id>0{
                            //用户-特定用户
                            role_find.find_access_user_by_res(
                                res_data.user_id, 
                                res_data.app_id,
                                user_id,
                                res_item.id,
                                op_item.id,
                            ).await;
                        }
                    }
                }
            } 
        }
        Ok(AccessRoleData{
            role_data: role_find.get_role_row().await?
        })
    }
}



#[derive(Clone)]
pub struct AccessRoleRow {
    pub role: RbacRoleModel,
    pub op_id: u64,
    pub res_id: u64,
    pub perm_id: u64,
    pub access_timeout: u64,
    pub access_user_id: u64,
}

pub struct AccessRoleData {
    role_data: Vec<AccessRoleRow>,
}

impl AccessRoleData{
    //获取系统禁止访问的角色列表
    pub fn get_system_exclude_role(&self,res_id:u64,op_id:u64)->Vec<&AccessRoleRow>{
        self.role_data.iter().filter(|e|{
            e.role.user_id==0 
            && RbacRoleResRange::Exclude.eq(e.role.res_range) 
            && e.res_id==res_id
            && e.op_id==op_id
            && (e.access_timeout==0||e.access_timeout>now_time().unwrap_or_default())
        }).collect::<Vec<_>>()
    }
    //获取系统允许全量访问的角色列表
    pub fn get_system_all_role(&self)->Vec<&AccessRoleRow>{
        self.role_data.iter().filter(|e|{
            e.role.user_id==0 
            && RbacRoleResRange::Any.eq(e.role.res_range)
            && (e.access_timeout==0||e.access_timeout>now_time().unwrap_or_default())
        }).collect::<Vec<_>>()
    }
    //获取系统允许部分访问的角色列表
    pub fn get_system_include_role(&self,res_id:u64,op_id:u64)->Vec<&AccessRoleRow>{
        self.role_data.iter().filter(|e|{
            e.role.user_id==0 
            && RbacRoleResRange::Include.eq(e.role.res_range)
            && e.res_id==res_id
            && e.op_id==op_id
            && (e.access_timeout==0||e.access_timeout>now_time().unwrap_or_default())
        }).collect::<Vec<_>>()
    }
    //获取用户禁止访问的角色列表
    pub fn get_user_exclude_role(&self,user_id:u64,res_id:u64,op_id:u64)->Vec<&AccessRoleRow>{
        self.role_data.iter().filter(|e|{
            e.role.user_id==user_id
            && RbacRoleResRange::Exclude.eq(e.role.res_range) 
            && e.res_id==res_id
            && e.op_id==op_id
            && (e.access_timeout==0||e.access_timeout>now_time().unwrap_or_default())
        }).collect::<Vec<_>>()
    }
    //获取用户允许全量访问的角色列表
    pub fn get_user_all_role(&self,user_id:u64)->Vec<&AccessRoleRow>{
        self.role_data.iter().filter(|e|{
            e.role.user_id==user_id
            && RbacRoleResRange::Any.eq(e.role.res_range)
            && (e.access_timeout==0||e.access_timeout>now_time().unwrap_or_default())
        }).collect::<Vec<_>>()
    }
    //获取用户允许部分访问的角色列表
    pub fn get_user_include_role(&self,user_id:u64,res_id:u64,op_id:u64)->Vec<&AccessRoleRow>{
        self.role_data.iter().filter(|e|{
            e.role.user_id==user_id
            && RbacRoleResRange::Include.eq(e.role.res_range)
            && e.res_id==res_id
            && e.op_id==op_id
            && (e.access_timeout==0||e.access_timeout>now_time().unwrap_or_default())
        }).collect::<Vec<_>>()
    }
}


struct RbacRoleFinder<'t>{
   role:&'t RbacRole,
   role_rows:Vec<AccessRoleRow>,
   sqls:Vec<(String,String)>
}
impl<'t> RbacRoleFinder<'t>{
    fn new(role:&'t RbacRole)->Self{
        Self{role,role_rows:vec![],sqls:vec![]}
    }
     // -----------------
     async fn find_access_user_res_all(
        &mut self,
        role_user_id:u64,
        role_user_app_id:u64,
        access_user_id:u64,
    ){   
        let key=self.role.cache().find_access_key_user_res_all(role_user_id,role_user_app_id, access_user_id);
        match self.role.cache_access.get(&key).await{
            Some(data) =>self.role_rows.extend(data),
            None =>{
                self.sqls.push((key,self.find_access_row_user_res_all(role_user_id,role_user_app_id,  access_user_id)))
            },
        }
    }
    fn find_access_row_user_res_all(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        access_user_id:u64,
    ) ->String{
        sql_format!(
            "select {} as cache_key,role.*,CONVERT(0,UNSIGNED) as op_id,CONVERT(0,UNSIGNED) as res_id ,
            CONVERT(0,UNSIGNED) as access_user_id, CONVERT(0,UNSIGNED) as access_timeout,CONVERT(0,UNSIGNED)  as perm_id
            from {} as role 
            join {} as role_user on role.id=role_user.role_id
            where role.status ={} and role.user_range={} 
            and role.user_id={role_user_id} and role.app_id={role_user_app_id} and role.res_range={} 
            and role_user.user_id={access_user_id} and role_user.status={}",
            self.role.cache().find_access_key_user_res_all(
                role_user_id,
                role_user_app_id,
                access_user_id,
            ),
            RbacRoleModel::table_name(),
            RbacRoleUserModel::table_name(),
            RbacRoleStatus::Enable as i8,
            RbacRoleUserRange::Custom as i8,
            RbacRoleResRange::Any as i8,
            RbacRoleUserStatus::Enable as i8,
        )           
    }
    async fn find_access_session_res_all(
        &mut self,
        role_user_id:u64,
        role_user_app_id:u64,
        session_role_key:&str,
    ){
        let key=self.role.cache().find_access_key_session_res_all(role_user_id,role_user_app_id,session_role_key);
        match self.role.cache_access.get(&key).await{
            Some(data) =>self.role_rows.extend(data),
            None =>{
                self.sqls.push((key,self.find_access_row_session_res_all(role_user_id,role_user_app_id,session_role_key)))
            },
        }
    }
    
    fn find_access_row_session_res_all(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        session_role_key:&str,
    ) ->String{
        sql_format!(
            "select {} as cache_key,role.*,CONVERT(0,UNSIGNED) as op_id,CONVERT(0,UNSIGNED) as res_id,
            CONVERT(0,UNSIGNED) as access_user_id, CONVERT(0,UNSIGNED) as access_timeout,CONVERT(0,UNSIGNED)  as perm_id
            from {} as role where role.status ={} and role.user_range={} 
            and role.user_id={role_user_id} and role.app_id={role_user_app_id}
            and role.res_range={} and role.role_key={session_role_key} ",
            self.role.cache().find_access_key_session_res_all(
                role_user_id,
                role_user_app_id,
                session_role_key,
            ),
            RbacRoleModel::table_name(),
            RbacRoleStatus::Enable as i8,
            RbacRoleUserRange::Session as i8,
            RbacRoleResRange::Any as i8,
        )           
    }
     // -----------------
    async fn find_access_session_by_res(
        &mut self,
        role_user_id:u64,
        role_user_app_id:u64,
        session_role_key:&str,
        res_id:u64,
        op_id:u64,
    ){   
        let key=self.role.cache().find_access_key_session_by_res(
            role_user_id, 
            role_user_app_id,
            session_role_key,
            RbacRoleResRange::Exclude as i8,
            res_id,
            op_id,
        );
         match self.role.cache_access.get(&key).await{
            Some(data) =>self.role_rows.extend(data),
            None =>{
                self.sqls.push((key,self.find_access_row_session_by_res(
                    role_user_id, 
                    role_user_app_id,
                    session_role_key,
                    RbacRoleResRange::Exclude,
                    res_id,
                    op_id,
                )));
            },
        }
        let key=self.role.cache().find_access_key_session_by_res(
            role_user_id, 
            role_user_app_id,
            session_role_key,
            RbacRoleResRange::Include as i8,
            res_id,
            op_id,
        );
        match self.role.cache_access.get(&key).await{
            Some(data) =>self.role_rows.extend(data),
            None =>{
                self.sqls.push((key,self.find_access_row_session_by_res(
                    role_user_id, 
                    role_user_app_id,
                    session_role_key,
                    RbacRoleResRange::Include,
                    res_id,
                    op_id,
                )));
            },
        }
    }
    fn find_access_row_session_by_res(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        session_role_key:&str,
        res_range:RbacRoleResRange,
        res_id:u64,
        op_id:u64,
    ) ->String{   
        sql_format!(
            "select {} as cache_key,role.*,perm.op_id,perm.res_id ,
            CONVERT(0,UNSIGNED) as access_user_id, CONVERT(0,UNSIGNED) as access_timeout,CONVERT(0,UNSIGNED)  as perm_id
            from {} as role join {} as perm on role.id=perm.role_id
            where role.status ={} and role.user_range={} and role.user_id={role_user_id} and role.app_id={role_user_app_id}
                and role.res_range={} and perm.op_id={op_id} and perm.res_id={res_id} 
                and prem.status={} and role.role_key={session_role_key} ",
            self.role.cache().find_access_key_session_by_res(
                role_user_id,
                role_user_app_id,
                session_role_key,
                res_range as i8,
                res_id,
                op_id,
            ),
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacRoleStatus::Enable as i8,
            RbacRoleUserRange::Session as i8,
            res_range as i8,
            RbacPermStatus::Enable as i8,
        )           
    }
      // -----------------
    async fn find_access_user_by_res(
        &mut self,
        role_user_id:u64,
        role_user_app_id:u64,
        access_user_id:u64,
        res_id:u64,
        op_id:u64,
    ) {   
        let key=self.role.cache().find_access_key_user_by_res(
            role_user_id,
            role_user_app_id,
            access_user_id,
            RbacRoleResRange::Exclude as i8,
            res_id,
            op_id,
        );
        match self.role.cache_access.get(&key).await{
            Some(data) =>self.role_rows.extend(data),
            None =>{
                self.sqls.push((key,self.find_access_row_user_by_res(
                    role_user_id,
                    role_user_app_id,
                    access_user_id,
                    RbacRoleResRange::Exclude,
                    res_id,
                    op_id,
                )))
            },
        }
        let key=self.role.cache().find_access_key_user_by_res(
            role_user_id,
            role_user_app_id,
            access_user_id,
            RbacRoleResRange::Include as i8,
            res_id,
            op_id,
        );
        match self.role.cache_access.get(&key).await{
            Some(data) =>self.role_rows.extend(data),
            None =>{
                self.sqls.push((key,self.find_access_row_user_by_res(
                    role_user_id,
                    role_user_app_id,
                    access_user_id,
                    RbacRoleResRange::Include,
                    res_id,
                    op_id,
                )))
            },
        }
    }
    fn find_access_row_user_by_res(
        &self,
        role_user_id:u64,
        role_user_app_id:u64,
        access_user_id:u64,
        res_range:RbacRoleResRange,
        res_id:u64,
        op_id:u64,
    ) ->String{   
        sql_format!(
            "select {} as cache_key,role.*,perm.op_id,perm.res_id,
            role_user.user_id as access_user_id,role_user.timeout as access_timeout,perm.id as perm_id
            from {} as role 
            join {} as perm on role.id=perm.role_id
            join {} as role_user on role.id=role_user.role_id where 
            role.status ={} and perm.status ={} and role_user.status ={}  
            and role.user_range={} and role.res_range={}  
            and perm.op_id={op_id} and perm.res_id={res_id} and role.user_id={role_user_id} and role.app_id={role_user_app_id}
            and role_user.user_id={access_user_id}",
            self.role.cache().find_access_key_user_by_res(
                role_user_id,
                role_user_app_id,
                access_user_id,
                res_range as i8,
                res_id,
                op_id,
            ),
            RbacRoleModel::table_name(),
            RbacPermModel::table_name(),
            RbacRoleUserModel::table_name(),
            RbacRoleStatus::Enable as i8,
            RbacPermStatus::Enable as i8,
            RbacRoleUserStatus::Enable as i8,
            RbacRoleUserRange::Custom as i8,
            res_range as i8,
        )           
    }
      // -----------------
    async fn find_access_row_by_sql(&self,sqls:&[&str])-> RbacResult<Vec<(String,AccessRoleRow)>>  {
        Ok(sqlx::query(&format!(
            "select * from ({}) as t",
            sqls.join(") union all (")
        ))
        .try_map(
            |row: sqlx::mysql::MySqlRow| match RbacRoleModel::from_row(&row) {
                Ok(role) => {
                    macro_rules! u64_row_get {
                        ($key:literal) => {
                            match row.try_get::<u64, &str>($key) {
                                Ok(id) => id,
                                Err(err) => {
                                    // dbg!("{:?}", err);
                                    error!(
                                        "find_access_row_by_sql get {} fail:{:?} on id :{}",
                                        $key,err, role.id
                                    );
                                    0
                                }
                            }
                        };
                    }
                    let op_id =u64_row_get!("op_id");
                    let res_id =u64_row_get!("res_id");
                    let perm_id =u64_row_get!("perm_id");
                    let access_timeout =u64_row_get!("access_timeout");
                    let access_user_id =u64_row_get!("access_user_id");
                    let cache_key= match row.try_get::<String, &str>("cache_key") {
                        Ok(id) => id,
                        Err(err) => {
                            return Err(err);
                        }
                    };
                    Ok((cache_key,AccessRoleRow {
                        role,
                        op_id,
                        res_id,
                        perm_id,
                        access_timeout,
                        access_user_id,
                    }))
                }
                Err(err) => Err(err),
            },
        )
        .fetch_all(&self.role.db)
        .await?)
    }
    async fn get_role_row(mut self)-> RbacResult<Vec<AccessRoleRow>>{
        if !self.sqls.is_empty() {
            let mut role_data=self.find_access_row_by_sql(&self.sqls.iter().map(|e|e.1.as_str()).collect::<Vec<_>>()).await?;
            for (key,_) in self.sqls{
                let mut set_tmp=Vec::with_capacity(role_data.len());
                let mut cache_data=Vec::with_capacity(role_data.len());
                for (tkey,tval) in role_data{
                    if tkey==key{
                        cache_data.push(tval);
                    }else{
                        set_tmp.push((tkey,tval));
                    }
                }
                self.role_rows.extend(cache_data.clone());
                self.role.cache_access.set(key, cache_data, 0).await;
                role_data=set_tmp;
            }
        }
        Ok(self.role_rows)
    }
}