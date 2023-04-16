use crate::dao::{SenderError, SenderResult};
use crate::model::{SenderCancelStatus, SenderKeyCancelModel, SenderKeyCancelModelRef, SenderType};
use lsys_core::now_time;

use sqlx::{MySql, Pool, Transaction};
use sqlx_model::{executor_option, Insert, Select, Update};

//短信任务记录
pub struct MessageCancel {
    db: Pool<sqlx::MySql>,
    send_type: SenderType,
}

impl MessageCancel {
    pub fn new(db: Pool<sqlx::MySql>, send_type: SenderType) -> Self {
        Self { db, send_type }
    }
    pub async fn add<'t>(
        &self,
        app_id: &u64,
        message_ids: &[u64],
        hk: &str,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> SenderResult<()> {
        let hk = hk.trim().to_string();
        if !hk.is_empty() {
            if hk.len() > 32 {
                return Err(SenderError::System("cancel key can't >32".to_owned()));
            }
            let add_time = now_time().unwrap_or_default();
            let sender_type = SenderType::Smser as i8;
            let mut idata = Vec::with_capacity(message_ids.len());
            for id in message_ids {
                idata.push(sqlx_model::model_option_set!(SenderKeyCancelModelRef, {
                    app_id:*app_id,
                    message_id:id,
                    cancel_key:hk,
                    sender_type:sender_type,
                    status:SenderCancelStatus::Init as i8,
                    cancel_user_id:0,
                    cancel_time:add_time,
                }));
            }
            executor_option!(
                {
                    Insert::<sqlx::MySql, SenderKeyCancelModel, _>::new_vec(idata)
                        .execute(db)
                        .await?;
                },
                transaction,
                &self.db,
                db
            );
        }
        Ok(())
    }
    //可取消发送的数据
    pub async fn cancel_data<'t>(
        &self,
        cancel_key: &str,
    ) -> SenderResult<Vec<SenderKeyCancelModel>> {
        let sender_type = self.send_type as i8;
        let status = SenderCancelStatus::Init as i8;

        let cancel_key = cancel_key.to_owned();
        let rows = Select::type_new::<SenderKeyCancelModel>()
            .fetch_all_by_where_call::<SenderKeyCancelModel, _, _>(
                "sender_type=? and cancel_key =? and status=?",
                |bind, _| bind.bind(sender_type).bind(cancel_key).bind(status),
                &self.db,
            )
            .await?;
        Ok(rows)
    }
    pub async fn cancel<'t>(
        &self,
        smsc: &SenderKeyCancelModel,
        user_id: &u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> SenderResult<()> {
        let change = sqlx_model::model_option_set!(SenderKeyCancelModelRef,{
            status:SenderCancelStatus::IsCancel as i8,
            cancel_user_id:user_id
        });
        executor_option!(
            {
                Update::<MySql, SenderKeyCancelModel, _>::new(change)
                    .execute_by_pk(smsc, db)
                    .await?;
            },
            transaction,
            &self.db,
            db
        );
        Ok(())
    }
}
