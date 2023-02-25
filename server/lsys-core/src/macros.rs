#[macro_export]
macro_rules! get_message {
    ($bundle:expr,$msg_id:expr,$default:expr)=>{
        {
            let fluent=$bundle;
            if !fluent.has_message($msg_id) {
                fluent.set_message($msg_id.to_string(),$default.to_string());
            }
            fluent.get_message($msg_id,None)
        }
    };
    ($bundle:expr,$msg_id:expr,$default:expr,[$($push_key:expr=>$push_val:expr),*])=>{
        {
            let fluent=$bundle;
            let mut args: fluent::FluentArgs =fluent::FluentArgs::new();
            $(
                args.set($push_key, $push_val);
            )*
            if !fluent.has_message($msg_id) {
                fluent.set_message($msg_id.to_string(),$default.to_string());
            }
            fluent.get_message($msg_id,Some(&args))
        }
    };
}

#[macro_export]
macro_rules! impl_dao_fetch_one_by_one {
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$where_id_name:ident,$where_sql:literal $(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, id: &$fetch_type) -> $result {
            let data = sqlx_model::Select::type_new::<$model>()
                .fetch_one_by_where::<$model, _>(
                    &sqlx_model::WhereOption::Where(sqlx_model::sql_format!($where_sql, $where_id_name = id.to_owned(),$($pat=$pav),*)),
                    &self.$db_field,
                )
                .await?;
            Ok(data)
        }
    };
}

#[macro_export]
macro_rules! impl_dao_fetch_vec_by_one {
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$where_id_name:ident,$where_sql:literal $(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, id: &$fetch_type) -> $result {
            let data = sqlx_model::Select::type_new::<$model>()
                .fetch_all_by_where::<$model, _>(
                    &sqlx_model::WhereOption::Where(sqlx_model::sql_format!($where_sql, $where_id_name = id.to_owned(),$($pat=$pav),*)),
                    &self.$db_field,
                )
                .await?;
            Ok(data)
        }
    };
}

#[macro_export]
macro_rules! impl_dao_fetch_map_by_vec {
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$field_name:ident,$where_id_name:ident ,$where_sql:literal$(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, ids: &[$fetch_type]) -> $result {
            if ids.is_empty() {
                return Ok(std::collections::HashMap::new());
            }
            use sqlx_model::SqlQuote;
            let data = sqlx_model::Select::type_new::<$model>()
                .fetch_all_by_where::<$model, _>(
                    &sqlx_model::WhereOption::Where(sqlx_model::sql_format!($where_sql, $where_id_name = ids,$($pat=$pav),*)),
                    &self.$db_field,
                )
                .await?;
            let mut hash = std::collections::HashMap::with_capacity(data.len());
            for data_ in data.into_iter() {
                hash.entry(data_.$field_name).or_insert(data_);
            }
            Ok(hash)
        }
    };
}

#[macro_export]
macro_rules! impl_dao_fetch_vec_by_vec {
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$field_name:ident,$where_id_name:ident,$where_sql:literal $(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, ids: &[$fetch_type]) -> $result {
            let mut hash = std::collections::HashMap::new();
            if ids.is_empty() {
                return Ok(hash);
            }
            use sqlx_model::SqlQuote;
            let data = sqlx_model::Select::type_new::<$model>()
                .fetch_all_by_where::<$model, _>(
                   & sqlx_model::WhereOption::Where(sqlx_model::sql_format!($where_sql, $where_id_name = ids,$($pat=$pav),*)),
                    &self.$db_field,
                )
                .await?;
            for data_ in data.into_iter() {
                hash.entry(data_.$field_name).or_insert(vec![]).push(data_);
            }
            Ok(hash)
        }
    };
}
