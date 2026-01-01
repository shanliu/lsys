### 短信消息详情查看接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | int | 是 | 消息ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.body | string | 消息内容(JSON字符串) |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/message_view
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "message_id":4
}

```

```json
{
  "response": {
    "body": "{\"code\":\"389506\",\"time\":\"300\"}"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```