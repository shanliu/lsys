### 短信登陆验证码发送

> 请求参数

> 验证码 `/captcha/login-sms/{captcha.key}`

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mobile | string | 是 | 手机号码 |
| area_code | string | 是 | 国际区号 |
| captcha.code | string | 是 | 验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数 

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.ttl | int | 验证码有效期(秒) |
| result.code | string | 状态码 |
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |


> 示例

```http
GET  /captcha/login-sms/8613800138000
```

```http
POST /api/auth/login/sms-send-code
Content-Type: application/json

{
    "mobile": "13800138001",
    "area_code": "86",
    "captcha":  {
        "code":"i3D",
        "key":"8613800138000"
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
