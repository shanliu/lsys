### 启用MFA绑定

为指定账号生成和启用MFA绑定，返回绑定后的记录ID

> payload参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| user_data | string | 是 | 用户数据 |
| secret | string | 是 | Base32编码的TOTP密钥 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.record_id | int | MFA绑定记录ID |
| response.user_data | string | 用户数据 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 说明

Base32编码密钥需要由客户端根据特定算法生成，用于生成TOTP验证码。启用后，用户需要使用此密钥配合认证器应用来生成验证码。

> 示例

```http
POST /rest/auth?method=mfa_enable
Content-type:application/json

{
   "user_data": "user@example.com",
   "secret": "JBSWY3DPEBLW64TMMQQ======="
}
```

```json
{
   "response": {
      "record_id": 12345,
      "user_data": "user@example.com"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
