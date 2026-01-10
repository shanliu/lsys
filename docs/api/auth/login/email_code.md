### 邮箱验证码登录

> 请求参数

> 验证码 `/captcha/login/{captcha.key}`

| 参数名         | 类型     | 必填 | 说明                |
| -------------- | -------- | ---- | ------------------- |
| email          | string   | 是   | 邮箱地址            |
| code           | string   | 是   | 邮箱验证码          |
| captcha.code   | string   | 是   | 图形验证码内容      |
| captcha.key    | string   | 是   | 图形验证码标识      |

> 响应参数

| 参数名                                      | 类型      | 说明                       |
| ------------------------------------------- | --------- | -------------------------- |
| response.mfa_token                           | string    | 需要MFA时返回的二次验证票据，拿到后调用 `/api/auth/login/mfa-verify` 完成登录 |
| response.auth_data.account_id               | int       | 账号ID                     |
| response.auth_data.empty_password           | int       | 是否无密码（0/1）          |
| response.auth_data.login_data.account_id    | int       | 账号ID                     |
| response.auth_data.login_data.change_time   | int       | 修改时间，单位：秒         |
| response.auth_data.login_data.confirm_time  | int       | 确认时间，单位：秒         |
| response.auth_data.login_data.email         | string    | 邮箱地址                   |
| response.auth_data.login_data.id            | int       | 登录数据ID                 |
| response.auth_data.login_data.status        | int       | 状态                       |
| response.auth_data.login_time               | int       | 登录有效期，单位：秒        |
| response.auth_data.login_type               | string    | 登录类型                   |
| response.auth_data.time_out                 | int       | 超时时间，单位：秒         |
| response.auth_data.user_id                  | int       | 用户ID                     |
| response.auth_data.user_nickname            | string    | 用户昵称                   |
| response.jwt                               | string    | JWT令牌                    |
| response.passwrod_timeout                   | int       | 密码超时标志（0/1）        |
| result.code                                 | int       | 结果码                     |
| result.message                              | string    | 结果信息                   |
| result.state                                | string    | 状态                       |

> 示例

```http
GET  /captcha/login/rustlang@qq.com

```

```http
POST /api/auth/login/email-code
Content-Type: application/json

{
    "email": "rustlang@qq.com",
    "code": "675188",
    "captcha":  {
        "code":"Pjn",
        "key":"rustlang@qq.com"
    }
}

```


```json
// 需要 MFA 时：
// {
//   "response": { "mfa_token": "..." },
//   "result": { "code": 200, "message": "ok", "state": "ok" }
// }

// MFA 不需要时：
{
  "response": {
    "auth_data": {
      "account_id": "5",
      "empty_password": "0",
      "login_data": {
        "account_id": "5",
        "change_time": "1748007185",
        "confirm_time": "0",
        "email": "rustlang@qq.com",
        "id": "1",
        "status": "2"
      },
      "login_time": "259200",
      "login_type": "email-code",
      "time_out": "1749477392",
      "user_id": "8",
      "user_nickname": "SHAN"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3NDk0NzczOTIsInRva2VuIjoiTUMwNExVcExTVkJDV2xsR1IweE1WVWxZUTFwQ1IxWk5VMGRNVVVsVlJVZERWVk5RTFRFM05EazBOemN6T1RJIiwiZGF0YSI6eyJhY2NvdW50X2lkIjo1LCJlbXB0eV9wYXNzd29yZCI6ZmFsc2UsImxvZ2luX2RhdGEiOnsiYWNjb3VudF9pZCI6NSwiY2hhbmdlX3RpbWUiOjE3NDgwMDcxODUsImNvbmZpcm1fdGltZSI6MCwiZW1haWwiOiJzaGFuLmxpdUBtc24uY29tIiwiaWQiOjEsInN0YXR1cyI6Mn0sImxvZ2luX3RpbWUiOjI1OTIwMCwibG9naW5fdHlwZSI6ImVtYWlsLWNvZGUiLCJ0aW1lX291dCI6MTc0OTQ3NzM5MiwidXNlcl9pZCI6OCwidXNlcl9uaWNrbmFtZSI6IlNIQU4ifX0.gJhaPmbUTQw7Quz_cJEP1r6m7qgydBcHGBzp2chBdIU",
    "passwrod_timeout": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
