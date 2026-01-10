### 使用手机号及密码登录

> 请求参数

> 验证码 `/captcha/login/{captcha.key}`

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mobile | string | 是 | 手机号 |
| area_code | string | 是 | 区号 |
| password | string | 是 | 密码 |
| captcha.code | string | 是 | 验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.mfa_token | string | 需要MFA时返回的二次验证票据，拿到后调用 `/api/auth/login/mfa-verify` 完成登录 |
| response.auth_data.account_id | string | 账户ID |
| response.auth_data.empty_password | string | 是否为空密码 |
| response.auth_data.login_data.account_id | string | 账户ID |
| response.auth_data.login_data.area_code | string | 区号 |
| response.auth_data.login_data.change_time | int | 变更时间(秒) |
| response.auth_data.login_data.confirm_time | int | 确认时间(秒) |
| response.auth_data.login_data.id | string | 登录数据ID |
| response.auth_data.login_data.mobile | string | 手机号 |
| response.auth_data.login_data.status | string | 状态码 |
| response.auth_data.login_time | int | 登录时长(秒) |
| response.auth_data.login_type | string | 登录类型 |
| response.auth_data.time_out | int | 超时时间(秒) |
| response.auth_data.user_id | string | 用户ID |
| response.auth_data.user_nickname | string | 用户昵称 |
| response.jwt | string | JWT令牌 |
| response.passwrod_timeout | string | 密码是否超时 |
| result.code | string | 返回码 |
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |

> 示例

```http
GET  /captcha/login/8613800138000

```

```http
POST /api/auth/login/sms
Content-Type: application/json

{
    "mobile": "13800138001",
    "area_code": "86",
    "password": "11323d1d",
    "captcha":  {
        "code":"rrw",
        "key":"8613800138000"
    }
}

```

```json
// 需要 MFA 时：
// {
//   "response": { "mfa_token": "..." },
//   "result": { "code": "200", "message": "ok", "state": "ok" }
// }

// MFA 不需要时：
{
  "response": {
    "auth_data": {
      "account_id": "6",
      "empty_password": "0",
      "login_data": {
        "account_id": "6",
        "area_code": "86",
        "change_time": "1749202398",
        "confirm_time": "0",
        "id": "1",
        "mobile": "13800138001",
        "status": "2"
      },
      "login_time": "259200",
      "login_type": "mobile",
      "time_out": "1749698923",
      "user_id": "12",
      "user_nickname": "SHAN11"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3NDk2OTg5MjMsInRva2VuIjoiTUMweE1pMVBTME5MVGtwV1YwaElRa1JYUmt0TlVVcE9Sa3RFV1VoYVFWaFlTbEpZVXkweE56UTVOams0T1RJeiIsImRhdGEiOnsiYWNjb3VudF9pZCI6NiwiZW1wdHlfcGFzc3dvcmQiOmZhbHNlLCJsb2dpbl9kYXRhIjp7ImFjY291bnRfaWQiOjYsImFyZWFfY29kZSI6Ijg2IiwiY2hhbmdlX3RpbWUiOjE3NDkyMDIzOTgsImNvbmZpcm1fdGltZSI6MCwiaWQiOjEsIm1vYmlsZSI6IjEzODAwMTM4MDAxIiwic3RhdHVzIjoyfSwibG9naW5fdGltZSI6MjU5MjAwLCJsb2dpbl90eXBlIjoibW9iaWxlLWNvZGUiLCJ0aW1lX291dCI6MTc0OTY5ODkyMywidXNlcl9pZCI6MTIsInVzZXJfbmlja25hbWUiOiJTSEFOMTEifX0.E-M6Z180H97GD7ehyfKM0AgiZsFXc-swcaTi4U9ptd4",
    "passwrod_timeout": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
