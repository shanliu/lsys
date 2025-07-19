### 邮箱登陆验证码发送

> 请求参数

> 验证码 `/captcha/login-email/{captcha.key}`

| 参数名         | 类型   | 必填 | 说明         |
| -------------- | ------ | ---- | ------------ |
| email          | string | 是   | 用户邮箱地址 |
| captcha.code   | string | 是   | 验证码内容   |
| captcha.key    | string | 是   | 验证码标识   |

> 响应参数

| 参数名               | 类型   | 说明                 |
| -------------------- | ------ | -------------------- |
| response.ttl         | int    | 验证码有效期(秒)     |
| result.code          | string | 结果代码             |
| result.message       | string | 结果信息             |
| result.state         | string | 状态                 |

> 示例


```http
GET  /captcha/login-email/rustlang@qq.com

```


```http
POST /api/auth/login/email-send-code
Content-Type: application/json

{
    "email": "rustlang@qq.com",
    "captcha":  {
        "code":"MzM",
        "key":"rustlang@qq.com"
    }
}

```

```json
{
  "response": {
    "ttl": "300"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
