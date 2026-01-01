### 删除邮件模板配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| tpl_config_id | int | 是 | 模板配置ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.num | string | 删除数量 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/tpl_config_del
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "tpl_config_id":18
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