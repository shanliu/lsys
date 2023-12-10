use crate::{
    dao::{SenderTaskResultItem, SenderTaskStatus},
    model::SenderSmsMessageModel,
};
#[allow(dead_code)]
fn sms_result_to_task(
    sms_arr: &[SenderSmsMessageModel],
    res: &[SendResultItem],
) -> Vec<SenderTaskResultItem> {
    sms_arr
        .iter()
        .map(|item| {
            res.iter()
                .find(|e| e.mobile == item.mobile)
                .map(|e| SenderTaskResultItem {
                    id: item.id,
                    status: match e.status {
                        SendStatus::Progress => SenderTaskStatus::Progress,
                        SendStatus::Completed => SenderTaskStatus::Completed,
                        SendStatus::Failed(retry) => SenderTaskStatus::Failed(retry),
                    },
                    message: e.message.to_owned(),
                    send_id: e.send_id.to_owned(),
                })
                .unwrap_or_else(|| SenderTaskResultItem {
                    id: item.id,
                    status: SenderTaskStatus::Failed(true),
                    message: "miss send".to_owned(),
                    send_id: "".to_owned(),
                })
        })
        .collect::<Vec<_>>()
}

#[cfg(feature = "sms-aliyun")]
mod sender_aliyun;
use lsys_lib_sms::{SendResultItem, SendStatus};
#[cfg(feature = "sms-aliyun")]
pub use sender_aliyun::*;

#[cfg(feature = "sms-huawei")]
mod sender_huawei;
#[cfg(feature = "sms-huawei")]
pub use sender_huawei::*;

#[cfg(feature = "sms-tencent")]
mod sender_tencent;
#[cfg(feature = "sms-tencent")]
pub use sender_tencent::*;

#[cfg(feature = "sms-jdcloud")]
mod sender_jdcloud;
#[cfg(feature = "sms-jdcloud")]
pub use sender_jdcloud::*;

#[cfg(feature = "sms-tencent")]
mod sender_netease;
#[cfg(feature = "sms-tencent")]
pub use sender_netease::*;

#[cfg(feature = "sms-cloopen")]
mod sender_cloopen;
#[cfg(feature = "sms-cloopen")]
pub use sender_cloopen::*;
