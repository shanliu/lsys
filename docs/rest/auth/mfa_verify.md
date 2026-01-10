### 验证MFA码

验证指定账号的MFA双因素认证码

> payload参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| user_data | string | 是 | 用户数据 |
| code | string | 是 | TOTP验证码，6-8位数字 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.user_data | string | 用户数据 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 说明

TOTP验证码由用户的认证器应用生成，有效期通常为30秒。验证失败会返回错误信息。

> 示例

```http
POST /rest/auth?method=mfa_verify
Content-type:application/json

{
   "user_data": "user@example.com",
   "code": "123456"
}
```

```json
{
   "response": {
      "user_data": "user@example.com"
   },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
