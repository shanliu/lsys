use std::collections::HashMap;

use crate::{dao::AccessResult, model::UserModel};

use super::AccessUser;
use lsys_core::db::{ModelTableName, SqlQuote};
use lsys_core::sql_format;
impl AccessUser {
    //通过登录数据查询用户
    pub async fn find_by_data(&self, app_id: u64, data: &str) -> AccessResult<UserModel> {
        let data = sqlx::query_as::<_, UserModel>(&sql_format!(
            "select * from {} where app_id={} and user_data={}",
            UserModel::table_name(),
            app_id,
            data
        ))
        .fetch_one(&self.db)
        .await?;
        Ok(data)
    }
    //通过ID获取用户
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        UserModel,
        AccessResult<UserModel>,
        id,
        "id = {id} "
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        UserModel,
        AccessResult<HashMap<u64, UserModel>>,
        id,
        ids,
        "id in ({ids}) "
    );
}
