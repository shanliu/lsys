### SMTP配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | 配置ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 配置列表 |
| response.data.change_time | int | 修改时间（秒） |
| response.data.change_user_id | string | 修改用户ID |
| response.data.email | string | 发件人邮箱 |
| response.data.hide_password | string | 隐藏密码 |
| response.data.hide_user | string | 隐藏用户名 |
| response.data.host | string | SMTP服务器地址 |
| response.data.id | string | 配置ID |
| response.data.name | string | 配置名称 |
| response.data.password | string | SMTP密码 |
| response.data.port | string | SMTP服务器端口 |
| response.data.timeout | string | 超时时间（秒） |
| response.data.tls_domain | string | TLS域名 |
| response.data.user | string | SMTP用户名 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/smtp_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "ids":null
}

```

```json
{
  "response": {
    "data": [
      {
        "change_time": "0",
        "change_user_id": "1",
        "email": "rustlang@qq.com",
        "hide_password": "cw**cj",
        "hide_user": "24**om",
        "host": "smtp.qq.com",
        "id": "4",
        "name": "111211",
        "password": "",
        "port": "465",
        "timeout": "30",
        "tls_domain": "",
        "user": "rustlang@qq.com"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```