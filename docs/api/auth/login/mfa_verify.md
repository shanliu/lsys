### MFA 二次验证登录

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mfa_token | string | 是 | 第一步登录返回的 MFA 票据 |
| code | string | 是 | TOTP 验证码（6位数字） |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.auth_data | object | 登录信息 |
| response.auth_data.app_data | object | 应用信息(可选) |
| response.auth_data.app_data.app_id | int | 应用ID |
| response.auth_data.app_data.client_id | string | 应用标识 |
| response.auth_data.app_data.app_name | string | 应用名称 |
| response.auth_data.app_data.change_time | int | 变更时间(秒) |
| response.auth_data.login_type | string | 登录类型 |
| response.auth_data.login_data | object | 登录数据 |
| response.auth_data.user_id | int | 用户ID |
| response.auth_data.user_nickname | string | 用户昵称 |
| response.auth_data.empty_password | int | 是否为空密码(0否1是) |
| response.auth_data.account_id | int | 账号ID |
| response.auth_data.time_out | int | 超时时间(秒) |
| response.auth_data.login_time | int | 登录时间(秒) |
| response.jwt | string | JWT令牌 |
| response.passwrod_timeout | int | 密码超时标志(0否1是) |
| result.code | int | 返回码 |
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/auth/login/mfa-verify
Content-Type: application/json

{
  "mfa_token": "...",
  "code": "123456"
}

```
