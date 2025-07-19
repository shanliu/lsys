### 取消邮件消息

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | string | 是 | 消息ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/message_cancel
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "message_id":"49"
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