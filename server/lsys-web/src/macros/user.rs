#[macro_export]
macro_rules! bind_vec_user_info_from_req{
    //$req_dao 请求对象
    //$data_vec 数据集
    //$field 数据集用户ID字段名 $out_field 输出字段
    //$privacy 隐藏敏感信息
    ($req_dao:expr,$data_vec:expr,[$($field:ident:$out_field:literal),*],$privacy:literal) => {
        {
            let mut user_id=vec![];
            $(
                user_id.extend($data_vec.iter().map(|e|e.$field).collect::<Vec<_>>());
            )*
            let user_data= $req_dao.web_dao.web_access.access_dao.user.cache().find_users_by_ids(&user_id).await?;
            $data_vec.iter().map(|e|{
                let mut val=json!(e);
                if let serde_json::Value::Object(ref mut map) = val {
                    $(
                       let add_val= if $privacy {
                             json!(user_data.get(&e.$field).map(|e|e.to_public()))
                        }else{
                            json!(user_data.get(&e.$field))
                        };
                        map.insert($out_field.to_string(),add_val);
                    )*
                }
                val
            }).collect::<Vec<_>>()
        }
    };
    ($req_dao:expr,$data_vec:expr,$field:ident,$privacy:literal) => {
        bind_vec_user_info_from_req!($req_dao,$data_vec,[$field:"user_data"],$privacy)
    };
    ($req_dao:expr,$data_vec:expr,$field:ident) => {
        bind_vec_user_info_from_req!($req_dao,$data_vec,[$field:"user_data"],true)
    };
}
