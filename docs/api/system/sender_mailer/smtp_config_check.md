### 检查SMTP配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| host | string | 是 | SMTP服务器地址 |
| port | int | 是 | SMTP服务器端口 |
| timeout | int | 是 | 超时时间（秒） |
| email | string | 是 | 发件人邮箱 |
| user | string | 是 | SMTP用户名 |
| password | string | 是 | SMTP密码 |
| tls_domain | string | 否 | TLS域名 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.status | string | 检查状态 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/smtp_config_check
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "host": "smtp.qq.com",
  "port": 465,
  "timeout": 30,
  "email": "rustlang@qq.com",
  "user": "rustlang@qq.com",
  "password": "",
  "tls_domain": ""
}
```

```json
{
  "response": {
    "status": "ok"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```