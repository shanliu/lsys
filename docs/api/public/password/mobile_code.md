### 手机重置密码验证码发送

> 请求参数 

> 验证码 `/captcha/reset-password-send-sms/{captcha.key}`

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mobile | string | 是 | 手机号码 |
| area_code | string | 是 | 国家区号 |
| captcha.code | string | 是 | 验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
GET  /captcha/reset-password-send-sms/8613800138000

```

> 示例

```http
POST /api/auth/password/mobile_code
Content-Type: application/json

{
    "mobile": "13800138001",
    "area_code":"86",
    "captcha": {
        "code":"pj7",
        "key":"8613800138000"
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
