### 编辑SMTP配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 是 | 配置ID |
| name | string | 是 | 配置名称 |
| host | string | 是 | SMTP服务器地址 |
| port | int | 是 | SMTP服务器端口 |
| timeout | int | 是 | 超时时间（秒） |
| email | string | 是 | 发件人邮箱 |
| user | string | 是 | SMTP用户名 |
| password | string | 是 | SMTP密码 |
| tls_domain | string | 否 | TLS域名 |
| branch_limit | int | 是 | 分支限制 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.num | string | 更新数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/smtp_config_edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "id":4,
  "name": "111we3",
  "host": "smtp.qq.com",
  "port": 465,
  "timeout": 30,
  "email": "rustlang@qq.com",
  "user": "rustlang@qq.com",
  "password": "",
  "tls_domain": "",
  "branch_limit": 1
}
```

```json
{
  "response": {
    "num": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```