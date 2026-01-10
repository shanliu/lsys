### 账号密码登录

> 请求参数

> 验证码 `/captcha/login/{captcha.key}`

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 用户名 |
| password | string | 是 | 密码 |
| captcha.code | string | 是 | 验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.mfa_token | string | 需要MFA时返回的二次验证票据，拿到后调用 `/api/auth/login/mfa-verify` 完成登录 |
| response.auth_data.account_id | string | 账号ID |
| response.auth_data.empty_password | string | 是否为空密码 |
| response.auth_data.login_data.account_id | string | 账号ID |
| response.auth_data.login_data.change_time | int | 更改时间(秒) |
| response.auth_data.login_data.id | string | 登录ID |
| response.auth_data.login_data.status | string | 状态 |
| response.auth_data.login_data.username | string | 用户名 |
| response.auth_data.login_time | int | 登录时长(秒) |
| response.auth_data.login_type | string | 登录类型 |
| response.auth_data.time_out | int | 超时时间(秒) |
| response.auth_data.user_id | string | 用户ID |
| response.auth_data.user_nickname | string | 用户昵称 |
| response.jwt | string | JWT令牌 |
| response.passwrod_timeout | string | 密码超时时间 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
### 获取验证码
GET  /captcha/login/aaaaa
```

> 示例

```http
POST /api/auth/login/name
Content-Type: application/json

{
    "name": "aaaaa",
    "password": "000000",
    "captcha":  {
        "code":"idx",
        "key":"aaaaa"
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
      "account_id": "1",
      "empty_password": "0",
      "login_data": {
        "account_id": "1",
        "change_time": "1748403663",
        "id": "1",
        "status": "1",
        "username": "aaaaa"
      },
      "login_time": "259200",
      "login_type": "name",
      "time_out": "1749264998",
      "user_id": "1",
      "user_nickname": "root"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3NDkyNjQ5OTgsInRva2VuIjoiTUMweExWRlhVVkZZVEZwWVUwSk1Va3RRVGxCVlZWQkRSRWhCV0ZWVlQwWk1TbE5PTFRFM05Ea3lOalE1T1RnIiwiZGF0YSI6eyJhY2NvdW50X2lkIjoxLCJlbXB0eV9wYXNzd29yZCI6ZmFsc2UsImxvZ2luX2RhdGEiOnsiYWNjb3VudF9pZCI6MSwiY2hhbmdlX3RpbWUiOjE3NDg0MDM2NjMsImlkIjoxLCJzdGF0dXMiOjEsInVzZXJuYW1lIjoiYWFhYWEifSwibG9naW5fdGltZSI6MjU5MjAwLCJsb2dpbl90eXBlIjoibmFtZSIsInRpbWVfb3V0IjoxNzQ5MjY0OTk4LCJ1c2VyX2lkIjoxLCJ1c2VyX25pY2tuYW1lIjoicm9vdCJ9fQ.ThuEmQshRNAZee43RwOgolI0gkw7xjgcsOXsnUBG3TM",
    "passwrod_timeout": "0"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```