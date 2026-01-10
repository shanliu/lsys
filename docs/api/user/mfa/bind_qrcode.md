### 获取MFA二维码

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.secret | string | Base32编码的Secret密钥 |
| response.otpauth_url | string | TOTP标准的otpauth协议URL |
| response.app_name | string | 应用名称 |
| response.len | string | 验证码长度 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/mfa/bind_qrcode
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}

```

```json
{
  "response": {
    "len":"6",
    "secret": "JBSWY3DPEBLW64TMMQ3G",
    "otpauth_url": "otpauth://totp/lsys:123?secret=JBSWY3DPEBLW64TMMQ3G&issuer=lsys",
    "app_name": "lsys"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
