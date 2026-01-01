### 查看短信内容

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | string | 是 | 短信消息ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.body | string | 短信内容(JSON字符串) |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/message_view
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
      "message_id":"1"
}
```

```json
{
  "response": {
    "body": "{\"code\":\"091286\",\"time\":\"120\"}"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

