### 外部账号通过应用完成登录

> token_data 由 `/rest/auth?method=do_login` 获取

> 请求参数

> 验证码 `/captcha/login/{captcha.key}`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| client_id | string | 是 | 应用ID |
| token_data | string | 是 | 登录令牌 | 
| captcha.code | string | 是 | 验证码内容 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| response.auth_data.account_id | string | 账号ID |
| response.auth_data.empty_password | string | 是否设置密码(1:未设置) |
| response.auth_data.login_data | object | 登录数据 |
| response.auth_data.login_time | int | 登录时间(秒) |
| response.auth_data.login_type | string | 登录类型 |
| response.auth_data.time_out | int | 超时时间(秒) |
| response.auth_data.user_id | string | 用户ID |
| response.auth_data.user_nickname | string | 用户昵称 |
| response.jwt | string | JWT令牌 |
| response.passwrod_timeout | string | 密码过期时间 |
| result.code | string | 响应代码 |  
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
GET  /captcha/login/app1_11111111

```

```http
POST /api/auth/login/app-code
Content-Type: application/json

{
    "client_id": "testapp1",
    "token_data": "3595aff6d32e74bffa93a42785dfef2f",
    "captcha": {
        "code":"bjB",
        "key":"app1_11111111"
    }
}
```

```json
{
  "response": {
    "auth_data": {
      "account_id": "0",
      "empty_password": "1",
      "login_data": {},
      "login_time": "9507",
      "login_type": "code",
      "time_out": "1749463469",
      "user_id": "17",
      "user_nickname": "xx"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3NDk0NjM0NjksInRva2VuIjoiTVMweE55MHpOVGsxWVdabU5tUXpNbVUzTkdKbVptRTVNMkUwTWpjNE5XUm1aV1l5WmkweE56UTVORFl6TkRZNSIsImRhdGEiOnsiYWNjb3VudF9pZCI6MCwiZW1wdHlfcGFzc3dvcmQiOnRydWUsImxvZ2luX2RhdGEiOnt9LCJsb2dpbl90aW1lIjo5NTA3LCJsb2dpbl90eXBlIjoiY29kZSIsInRpbWVfb3V0IjoxNzQ5NDYzNDY5LCJ1c2VyX2lkIjoxNywidXNlcl9uaWNrbmFtZSI6Inh4In19.grtWyUB_015qP9s6_eLZU8kiXw2XT0e-I9WdkZF9npU",
    "passwrod_timeout": "0"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
