### 发送重置密码邮箱验证码

> 请求参数说明

> 验证码 `/captcha/reset-password-send-mail/{captcha.key}` 

| 参数名 | 类型 | 是否必填 | 描述 |
|--------|------|----------|------|
| email | string | 是 | 邮箱地址 |
| captcha.code | string | 是 | 验证码 |
| captcha.key | string | 是 | 验证码key(邮箱地址) |

> 响应参数说明

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 返回消息 |
| result.state | string | 状态说明 |


> 示例

```http
### email password reset cpatcha
GET  /captcha/reset-password-send-mail/rustlang@qq.com
```


> 示例

```http
### email password reset
POST /api/auth/password/email_code
Content-Type: application/json

{
    "email": "rustlang@qq.com",
    "captcha": {
        "code":"j7g",
        "key":"rustlang@qq.com"
    }
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
