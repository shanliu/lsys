### 使用邮箱及密码登录

> 请求参数

> 验证码 `/captcha/login/{captcha.key}`

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| email | string | 是 | 邮箱地址 |
| password | string | 是 | 密码 |
| captcha.code | string | 是 | 验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.auth_data.account_id | string | 账户ID |
| response.auth_data.empty_password | string | 是否为空密码 |
| response.auth_data.login_data.account_id | string | 登录账户ID |
| response.auth_data.login_data.change_time | int | 修改时间 |
| response.auth_data.login_data.confirm_time | int | 确认时间 |
| response.auth_data.login_data.email | string | 邮箱地址 |
| response.auth_data.login_data.id | string | 登录ID |
| response.auth_data.login_data.status | string | 状态码 |
| response.auth_data.login_time | int | 登录时长 |
| response.auth_data.login_type | string | 登录类型 |
| response.auth_data.time_out | int | 超时时间 |
| response.auth_data.user_id | string | 用户ID |
| response.auth_data.user_nickname | string | 用户昵称 |
| response.jwt | string | JWT令牌 |
| response.passwrod_timeout | string | 密码是否超时 |
| result.code | string | 结果状态码 |
| result.message | string | 结果消息 |
| result.state | string | 结果状态 |

> 示例

```http
GET  /captcha/login/rustlang@qq.com

```


> 示例

```http
POST /api/auth/login/email
Content-Type: application/json

{
    "email": "rustlang@qq.com",
    "password": "121qqq121q",
     "captcha":  {
        "code":"qjp",
        "key":"rustlang@qq.com"
    }
}

```

```json
{
  "response": {
    "auth_data": {
      "account_id": "5",
      "empty_password": "0",
      "login_data": {
        "account_id": "5",
        "change_time": "1749199820",
        "confirm_time": "0",
        "email": "rustlang@qq.com",
        "id": "1",
        "status": "2"
      },
      "login_time": "259200",
      "login_type": "email",
      "time_out": "1749698346",
      "user_id": "11",
      "user_nickname": "SHAN"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3NDk2OTgzNDYsInRva2VuIjoiTUMweE1TMUpRVXRFUWtOV1FrOU9XVTFaUVZaVFMxcEZSVUZUV0VsV1EwZElSa2RMVWkweE56UTVOams0TXpRMiIsImRhdGEiOnsiYWNjb3VudF9pZCI6NSwiZW1wdHlfcGFzc3dvcmQiOmZhbHNlLCJsb2dpbl9kYXRhIjp7ImFjY291bnRfaWQiOjUsImNoYW5nZV90aW1lIjoxNzQ5MTk5ODIwLCJjb25maXJtX3RpbWUiOjAsImVtYWlsIjoic2hhbi5saXVAbXNuLmNvbSIsImlkIjoxLCJzdGF0dXMiOjJ9LCJsb2dpbl90aW1lIjoyNTkyMDAsImxvZ2luX3R5cGUiOiJlbWFpbCIsInRpbWVfb3V0IjoxNzQ5Njk4MzQ2LCJ1c2VyX2lkIjoxMSwidXNlcl9uaWNrbmFtZSI6IlNIQU4ifX0.bAHJ74tBftyDJCYVRtusBRvfL7hd5mms7gguQqJa4m4",
    "passwrod_timeout": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
