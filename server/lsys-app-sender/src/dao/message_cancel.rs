use crate::dao::SenderResult;
use crate::model::{SenderMessageCancelModel, SenderMessageCancelModelRef, SenderType};
use lsys_core::now_time;

use sqlx::{Pool, Transaction};
use sqlx_model::{executor_option, Insert};

//短信取消发送公共代码

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
        sender_body_id: &u64,
        message_ids: &[u64],
        cancel_user_id: &u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> SenderResult<()> {
        if message_ids.is_empty(){
            return Ok(());
        }
        let add_time = now_time().unwrap_or_default();
        let sender_type = self.send_type as i8;

        let mut idata = Vec::with_capacity(message_ids.len());
        for id in message_ids {
            idata.push(sqlx_model::model_option_set!(SenderMessageCancelModelRef, {
                app_id:*app_id,
                sender_body_id:*sender_body_id,
                sender_message_id:id,
                sender_type:sender_type,
                cancel_user_id:*cancel_user_id,
                cancel_time:add_time,
            }));
        }
        executor_option!(
            {
                Insert::<sqlx::MySql, SenderMessageCancelModel, _>::new_vec(idata)
                    .execute(db)
                    .await?;
            },
            transaction,
            &self.db,
            db
        );
        Ok(())
    }
}
