//绑定用户信息
#[macro_export]
macro_rules! bind_user_info {
    ($access_dao:expr,$data_vec:expr,[$($field:ident:$out_field:literal),*],$privacy:literal) => {
        {
            let mut user_id=vec![];
            $(
                user_id.extend($data_vec.iter().map(|e|e.$field).collect::<Vec<_>>());
            )*
            let user_data= $access_dao.cache().find_users_by_ids(&user_id).await?;
            $data_vec.iter().map(|e|{
                let mut val=json!(e);
                if !val.is_object() {
                    val
                }else{
                    $(
                        if $privacy {
                            val[$out_field]=json!(user_data.get(&e.$field).map(|e|e.to_public()));
                        }else{
                            val[$out_field]=json!(user_data.get(&e.$field));
                        }
                    )*
                    val
                }
            }).collect::<Vec<_>>()
        }
    };
    ($user_dao:expr,$data_vec:expr,$field:ident,$privacy:literal) => {
        bind_user_info!($user_dao,$data_vec,[$field:"user_data"],$privacy)
    };
    ($user_dao:expr,$data_vec:expr,$field:ident) => {
        bind_user_info!($user_dao,$data_vec,[$field:"user_data"],true)
    };
}
#[macro_export]
macro_rules! bind_user_info_from_req {
    ($req_dao:expr,$data_vec:expr,[$($field:ident:$out_field:literal),*],$privacy:literal) => {
        $crate::bind_user_info!($req_dao.web_dao.web_access.access_dao.user,$data_vec,[$($field:$out_field),*],$privacy)
    };
    ($req_dao:expr,$data_vec:expr,$field:ident,$privacy:literal) => {
        bind_user_info_from_req!($req_dao,$data_vec,[$field:"user_data"],$privacy)
    };
    ($req_dao:expr,$data_vec:expr,$field:ident) => {
        bind_user_info_from_req!($req_dao,$data_vec,[$field:"user_data"],true)
    };
}
