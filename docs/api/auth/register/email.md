### 完成邮箱注册

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| nikename | string | 是 | 用户昵称 |
| email | string | 是 | 邮箱地址 |
| code | string | 是 | 验证码,由 /register/email-code 发送 |
| password | string | 是 | 密码 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 用户ID |
| result.code | string | 状态码 |
| result.message | string | 响应信息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/auth/register/email
Content-Type: application/json

{
    "nikename":"SHAN",
    "email": "rustlang@qq.com",
    "code": "213983",
    "password":"121qqq121q"
}
```

```json
{
  "response": {
    "id": "5"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
