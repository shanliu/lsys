use std::collections::HashSet;
use std::sync::Arc;

use super::SenderResult;
use lsys_setting::dao::{MultipleSetting, SettingData, SettingDecode, SettingEncode};
use sqlx::{FromRow, MySql, Pool};
use sqlx_model::{sql_format, ModelTableField, ModelTableName};
use sqlx_model::{Select, SqlQuote};

//短信任务记录
pub struct AppConfigReader<M, C>
where
    C: SettingDecode + SettingEncode,
    for<'t> M: FromRow<'t, sqlx::mysql::MySqlRow>
        + Send
        + Unpin
        + ModelTableName
        + ModelTableField<MySql>
        + Clone,
{
    db: Pool<sqlx::MySql>,
    setting: Arc<MultipleSetting>,
    marker_model: std::marker::PhantomData<M>,
    marker_config: std::marker::PhantomData<C>,
}

impl<M, C> AppConfigReader<M, C>
where
    C: SettingDecode + SettingEncode,
    for<'t> M: FromRow<'t, sqlx::mysql::MySqlRow>
        + Send
        + Unpin
        + ModelTableName
        + ModelTableField<MySql>
        + Clone,
{
    pub fn new(db: Pool<sqlx::MySql>, setting: Arc<MultipleSetting>) -> Self {
        Self {
            db,
            marker_model: std::marker::PhantomData,
            marker_config: std::marker::PhantomData,
            setting,
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(db, find_by_id, u64, M, SenderResult<M>, id, "id={id}");
    #[allow(clippy::too_many_arguments)]
    pub async fn list_config(
        &self,
        id: &Option<u64>,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<i8>,
        sql_where: Option<String>,
        model_to_id: &impl Fn(&M) -> u64,
    ) -> SenderResult<Vec<(M, SettingData<C>)>> {
        let mut sqlwhere = vec![sql_format!("status ={}", status)];
        if let Some(aid) = id {
            sqlwhere.push(sql_format!("id = {}  ", aid));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(tpl) = tpl_id {
            sqlwhere.push(sql_format!("tpl_id={} ", tpl));
        }
        if let Some(s) = sql_where {
            sqlwhere.push(s);
        }
        let sql = format!("{}  order by id desc", sqlwhere.join(" and "));
        let res = Select::type_new::<M>()
            .fetch_all_by_where::<M, _>(&sqlx_model::WhereOption::Where(sql), &self.db)
            .await?;
        if res.is_empty() {
            return Ok(vec![]);
        }
        let ids = res
            .iter()
            .map(model_to_id)
            .collect::<HashSet<u64>>()
            .iter()
            .map(|e| e.to_owned())
            .collect::<Vec<u64>>();
        let ali_res = self
            .setting
            .list_data::<C>(&None, &Some(ids), &None)
            .await?;
        if ali_res.is_empty() {
            return Ok(vec![]);
        }
        let out = ali_res
            .into_iter()
            .filter_map(|e| {
                let out = res.iter().find(|r| e.model().id == model_to_id(r));
                out.map(|s| (s.to_owned(), e))
            })
            .collect::<Vec<(_, _)>>();
        Ok(out)
    }
}
