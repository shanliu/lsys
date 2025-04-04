use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account")]
pub struct AccountModel {
    /// 用户ID
    #[sqlx(default)]
    pub id: u64,

    /// 昵称
    #[sqlx(default)]
    pub nickname: String,

    /// 1bit 是否激活 2bit 是否屏蔽  default:  1
    #[sqlx(default)]
    pub status: i8,

    /// 密码ID  default:  0
    #[sqlx(default)]
    pub password_id: u64,

    /// 是否启用用户名  default:  0
    #[sqlx(default)]
    pub use_name: i8,

    /// 绑定邮箱数量  default:  0
    #[sqlx(default)]
    pub email_count: u32,

    /// 绑定手机数量  default:  0
    #[sqlx(default)]
    pub mobile_count: u32,

    /// 绑定外部账号数量  default:  0
    #[sqlx(default)]
    pub external_count: u32,

    /// 收货地址数量  default:  0
    #[sqlx(default)]
    pub address_count: u32,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 状态确认时间,激活时间
    #[sqlx(default)]
    pub confirm_time: u64,

    /// 最后更改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_address")]
pub struct AccountAddressModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户id
    #[sqlx(default)]
    pub account_id: u64,

    /// 国家代码
    #[sqlx(default)]
    pub country_code: String,

    /// 地址代码
    #[sqlx(default)]
    pub address_code: String,

    /// 地址信息,冗余,显示用
    #[sqlx(default)]
    pub address_info: String,

    /// 地址详细
    #[sqlx(default)]
    pub address_detail: String,

    /// 姓名
    #[sqlx(default)]
    pub name: String,

    /// 电话
    #[sqlx(default)]
    pub mobile: String,

    /// 是否启用
    #[sqlx(default)]
    pub status: i8,

    /// 最后更改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_email")]
pub struct AccountEmailModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub account_id: u64,

    /// 邮箱
    #[sqlx(default)]
    pub email: String,

    /// 绑定状态1正常 2待验证 3关闭  default:  0
    #[sqlx(default)]
    pub status: i8,

    /// 确认时间
    #[sqlx(default)]
    pub confirm_time: u64,

    /// 最后更改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_external")]
pub struct AccountExternalModel {
    #[sqlx(default)]
    pub id: u64,

    /// 使用配置名
    #[sqlx(default)]
    pub config_name: String,

    /// 用户ID
    #[sqlx(default)]
    pub account_id: u64,

    /// 类型 1 微信
    #[sqlx(default)]
    pub external_type: String,

    /// 其他网站用户标识
    #[sqlx(default)]
    pub external_id: String,

    /// 其他网站用户名
    #[sqlx(default)]
    pub external_name: String,

    /// 1 男 2女  default:  0
    #[sqlx(default)]
    pub external_gender: String,

    /// 其他网站用户链接
    #[sqlx(default)]
    pub external_link: String,

    /// 其他网站用户头像
    #[sqlx(default)]
    pub external_pic: String,

    /// 其他网站用户显示名
    #[sqlx(default)]
    pub external_nikename: String,

    /// 是否标注为删除 0 表示删除 1 表示正常  default:  1
    #[sqlx(default)]
    pub status: i8,

    /// 登录后的token
    #[sqlx(default)]
    pub token_data: String,

    /// 登录token超时
    #[sqlx(default)]
    pub token_timeout: u64,

    /// 更新时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_info")]
pub struct AccountInfoModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户id
    #[sqlx(default)]
    pub account_id: u64,

    /// 性别 1 男 2 女  default:  0
    #[sqlx(default)]
    pub gender: i32,

    /// 头像地址
    #[sqlx(default)]
    pub headimg: String,

    /// 生日
    #[sqlx(default)]
    pub birthday: String,

    /// 注册IP
    #[sqlx(default)]
    pub reg_ip: String,

    /// 注册来源
    #[sqlx(default)]
    pub reg_from: String,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_login")]
pub struct AccountLoginModel {
    #[sqlx(default)]
    pub id: u64,

    /// 登录方式 ID密码登录 1 账号密码登录2 邮箱登录3 手机登录4 手机验证码登录5 外部账号登录6 链接登录7
    #[sqlx(default)]
    pub login_type: String,

    /// 尝试登录账号
    #[sqlx(default)]
    pub login_account: String,

    /// 登陆者IP
    #[sqlx(default)]
    pub login_ip: String,

    /// IP对应城市
    #[sqlx(default)]
    pub login_city: String,

    /// 尝试登录账号对应用户ID  default:  0
    #[sqlx(default)]
    pub account_id: u64,

    /// 是否登录成功  default:  0
    #[sqlx(default)]
    pub is_login: i8,

    /// 登录错误消息
    #[sqlx(default)]
    pub login_msg: String,

    /// 登录时间
    #[sqlx(default)]
    pub add_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_mobile")]
pub struct AccountMobileModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub account_id: u64,

    /// 电话区号
    #[sqlx(default)]
    pub area_code: String,

    /// 手机号
    #[sqlx(default)]
    pub mobile: String,

    /// 绑定状态1正常 2待验证 3关闭  default:  0
    #[sqlx(default)]
    pub status: i8,

    /// 确认时间
    #[sqlx(default)]
    pub confirm_time: u64,

    /// 最后更改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_name")]
pub struct AccountNameModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户id
    #[sqlx(default)]
    pub account_id: u64,

    /// 登录用户名
    #[sqlx(default)]
    pub username: String,

    /// 最后更新时间
    #[sqlx(default)]
    pub change_time: u64,

    /// 绑定状态1正常 2待验证 3关闭  default:  0
    #[sqlx(default)]
    pub status: i8,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_password")]
pub struct AccountPasswordModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub account_id: u64,

    /// 密码
    #[sqlx(default)]
    pub password: String,

    /// 绑定时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 停用时间  default:  0
    #[sqlx(default)]
    pub disable_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "account_index")]
pub struct AccountIndexModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub account_id: u64,

    /// 分类
    #[sqlx(default)]
    pub index_cat: u8,

    /// 索引数据
    #[sqlx(default)]
    pub index_data: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 最后更新时间
    #[sqlx(default)]
    pub change_time: u64,
}
