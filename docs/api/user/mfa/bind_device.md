### 绑定MFA设备

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| secret | string | 是 | Base32编码的Secret密钥 |
| code | string | 是 | MFA设备显示的TOTP验证码（6位数字） |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/mfa/bind_device
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "secret": "JBSWY3DPEBLW64TMMQ3G",
  "code": "123456"
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
