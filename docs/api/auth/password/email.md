### 邮箱验证码重置密码

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| email | string | 邮箱地址 |
| code | string | 验证码 |
| new_password | string | 新密码 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
### email password reset do
POST /api/auth/password/email
Content-Type: application/json

{
    "email": "rustlang@qq.com",
    "code": "594238",
    "new_password": "bb13579"
}
```


```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
