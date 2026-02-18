use crate::dao::SenderResult;
use crate::model::{SenderMessageCancelModel, SenderType};
use lsys_core::now_time;

use lsys_core::db::BatchInsert;
use lsys_core::db::Insert;
use lsys_core::db_option_executor;
use sqlx::{Pool, Transaction};
//短信取消发送公共代码

pub struct MessageCancel {
    db: Pool<sqlx::MySql>,
    send_type: SenderType,
}

impl MessageCancel {
    pub fn new(db: Pool<sqlx::MySql>, send_type: SenderType) -> Self {
        Self { db, send_type }
    }
    pub async fn add(
        &self,
        app_id: u64,
        sender_body_id: u64,
        message_ids: &[u64],
        cancel_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> SenderResult<()> {
        if message_ids.is_empty() {
            return Ok(());
        }
        let add_time = now_time().unwrap_or_default();
        let sender_type = self.send_type as i8;

        let mut batch = BatchInsert::<SenderMessageCancelModel>::with_capacity(message_ids.len());
        for id in message_ids {
            batch = batch.push(
                Insert::<SenderMessageCancelModel>::new()
                    .set(SenderMessageCancelModel::APP_ID, app_id)
                    .set(SenderMessageCancelModel::SENDER_BODY_ID, sender_body_id)
                    .set(SenderMessageCancelModel::SENDER_MESSAGE_ID, *id)
                    .set(SenderMessageCancelModel::SENDER_TYPE, sender_type)
                    .set(SenderMessageCancelModel::CANCEL_USER_ID, cancel_user_id)
                    .set(SenderMessageCancelModel::CANCEL_TIME, add_time),
            );
        }
        db_option_executor!(
            db,
            {
                batch.execute(db.as_executor()).await?;
            },
            transaction,
            &self.db
        );
        Ok(())
    }
}
