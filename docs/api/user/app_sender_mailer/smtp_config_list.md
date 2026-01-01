### 获取SMTP配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | SMTP配置ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.email | string | 邮箱地址 |
| response.data.id | string | 配置ID |
| response.data.name | string | 配置名称 |
| response.data.user | string | 用户名 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/smtp_config_list
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
        "email": "rustlang@qq.com",
        "id": "4",
        "name": "111we3",
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