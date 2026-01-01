### 发送邮件消息

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| tpl_key | string | 是 | 模板标识 |
| to | array | 是 | 收件人邮箱列表 |
| data | object | 是 | 模板变量数据 |
| reply | string | 否 | 回复邮箱 |
| send_time | string | 否 | 定时发送时间(格式:YYYY-MM-DD HH:mm:ss) |
| max_try | int | 否 | 最大重试次数 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/message_send
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
   "tpl_key": "ddddd",
    "to":["rustlang@qq.com"],
    "data":{"code":"2323","aa":"232323"},
    "reply":null,
    "send_time":"2025-05-27 18:55:55",
    "max_try":1
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