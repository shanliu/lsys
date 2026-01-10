### 短信验证码登录

> 请求参数

> 验证码 `/captcha/login/{captcha.key}`


| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mobile | string | 是 | 手机号码 |
| area_code | string | 是 | 区号 |
| code | string | 是 | 短信验证码 |
| captcha.code | string | 是 | 图形验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.mfa_token | string | 需要MFA时返回的二次验证票据，拿到后调用 `/api/auth/login/mfa-verify` 完成登录 |
| response.auth_data.account_id | string | 账户ID |
| response.auth_data.empty_password | string | 是否为空密码(0否1是) |
| response.auth_data.login_data.account_id | string | 登录账户ID |
| response.auth_data.login_data.area_code | string | 区号 |
| response.auth_data.login_data.change_time | int | 修改时间(秒) |
| response.auth_data.login_data.confirm_time | int | 确认时间(秒) |
| response.auth_data.login_data.id | string | 登录数据ID |
| response.auth_data.login_data.mobile | string | 手机号码 |
| response.auth_data.login_data.status | string | 状态码 |
| response.auth_data.login_time | int | 登录时长(秒) |
| response.auth_data.login_type | string | 登录类型 |
| response.auth_data.time_out | int | 超时时间(秒) |
| response.auth_data.user_id | string | 用户ID |
| response.auth_data.user_nickname | string | 用户昵称 |
| response.jwt | string | JWT令牌 |
| response.passwrod_timeout | string | 密码超时标志(0否1是) |
| result.code | string | 状态码 |
| result.message | string | 状态消息 |
| result.state | string | 状态标识 |


> 示例

```http
GET  /captcha/login/1380013800086

```

```http
POST /api/auth/login/sms-code
Content-Type: application/json

{
    "mobile": "13800138001",
    "area_code": "86",
    "code": "560259",
     "captcha":  {
        "code":"rk3",
        "key":"1380013800086"
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
      "login_type": "mobile-code",
      "time_out": "1749698793",
      "user_id": "12",
      "user_nickname": "SHAN11"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3NDk2OTg3OTMsInRva2VuIjoiTUMweE1pMVhRMUJhU2xkRlNVUkZVVkZQVlVkSVQxWkxWMDlUVEZoT1NGWkVVRmhYVHkweE56UTVOams0TnpreiIsImRhdGEiOnsiYWNjb3VudF9pZCI6NiwiZW1wdHlfcGFzc3dvcmQiOmZhbHNlLCJsb2dpbl9kYXRhIjp7ImFjY291bnRfaWQiOjYsImFyZWFfY29kZSI6Ijg2IiwiY2hhbmdlX3RpbWUiOjE3NDkyMDIzOTgsImNvbmZpcm1fdGltZSI6MCwiaWQiOjEsIm1vYmlsZSI6IjEzODAwMTM4MDAxIiwic3RhdHVzIjoyfSwibG9naW5fdGltZSI6MjU5MjAwLCJsb2dpbl90eXBlIjoibW9iaWxlLWNvZGUiLCJ0aW1lX291dCI6MTc0OTY5ODc5MywidXNlcl9pZCI6MTIsInVzZXJfbmlja25hbWUiOiJTSEFOMTEifX0.-G7Z2-K4ltCDb3JBHOA7TX82TA-BVZDvwMsV1RGnxpw",
    "passwrod_timeout": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```