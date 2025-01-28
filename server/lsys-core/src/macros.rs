#[macro_export]
macro_rules! impl_dao_fetch_one_by_one {
    //通过一个值查找一个记录 一对一关系
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$where_id_name:ident,$where_sql:literal $(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, $where_id_name: &$fetch_type) -> $result {
            use $crate::db::SqlQuote;
            use $crate::db::SqlExpr;
            use $crate::db::ModelTableName;
            let data = sqlx::query_as::<_, $model>(&$crate::sql_format!(
                "select * from {} where {} ",
                <$model>::table_name(),
                SqlExpr($crate::sql_format!($where_sql, $where_id_name = $where_id_name.to_owned(),$($pat=$pav),*))
            ))
            .fetch_one(&self.$db_field)
            .await?;
            Ok(data)
        }
    };
}

#[macro_export]
macro_rules! impl_dao_fetch_vec_by_one {
    //通过一个值查找一批值 一对多关系
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$where_id_name:ident,$where_sql:literal $(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, id: &$fetch_type) -> $result {
            use $crate::db::SqlQuote;
            use $crate::db::SqlExpr;
            use $crate::db::ModelTableName;
            let data = sqlx::query_as::<_, $model>(&$crate::sql_format!(
                "select * from {} where {} ",
                <$model>::table_name(),
                SqlExpr($crate::sql_format!($where_sql, $where_id_name = id.to_owned(),$($pat=$pav),*))
            ))
            .fetch_all(&self.$db_field)
            .await?;
            Ok(data)
        }
    };
}

#[macro_export]
macro_rules! impl_dao_fetch_map_by_vec {
    //通过一批值查找一批值 一对一关系
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$field_name:ident,$where_id_name:ident ,$where_sql:literal$(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, ids: &[$fetch_type]) -> $result {
            if ids.is_empty() {
                return Ok(std::collections::HashMap::new());
            }
            use $crate::db::SqlQuote;
            use $crate::db::SqlExpr;
            use $crate::db::ModelTableName;
            let data = sqlx::query_as::<_, $model>(&$crate::sql_format!(
                "select * from {} where {} ",
                <$model>::table_name(),
                SqlExpr($crate::sql_format!($where_sql, $where_id_name = ids,$($pat=$pav),*))
            ))
            .fetch_all(&self.$db_field)
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
    //通过一批值查找一批值 一对多关系
    ($db_field:ident,$fn:ident,$fetch_type:ty,$model:ty,$result:ty,$field_name:ident,$where_id_name:ident,$where_sql:literal $(,$pat:ident=$pav:expr),*) => {
        pub async fn $fn(&self, ids: &[$fetch_type]) -> $result {
            let mut hash = std::collections::HashMap::new();
            if ids.is_empty() {
                return Ok(hash);
            }
            use $crate::db::SqlQuote;
            use $crate::db::SqlExpr;
            use $crate::db::ModelTableName;
            let data = sqlx::query_as::<_, $model>(&$crate::sql_format!(
                "select * from {} where {} ",
                <$model>::table_name(),
                SqlExpr($crate::sql_format!($where_sql, $where_id_name = ids,$($pat=$pav),*))
            ))
            .fetch_all(&self.$db_field)
            .await?;
            for data_ in data.into_iter() {
                hash.entry(data_.$field_name).or_insert(vec![]).push(data_);
            }
            Ok(hash)
        }
    };
}
