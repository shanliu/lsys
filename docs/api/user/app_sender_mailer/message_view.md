### 查看邮件消息内容

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | string | 是 | 消息ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.body | string | 消息内容(JSON字符串) |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/message_view
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
      "message_id":"55"
}
```

```json
{
  "response": {
    "body": "{\"aa\":\"232323\",\"code\":\"2323\"}"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```