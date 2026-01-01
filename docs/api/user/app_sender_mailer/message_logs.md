### 获取邮件发送日志列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | string | 是 | 消息ID |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.app_id | string | 应用ID |
| response.data.create_time | string | 创建时间(秒) |
| response.data.executor_type | string | 执行器类型 |
| response.data.id | string | 日志ID |
| response.data.log_type | string | 日志类型 |
| response.data.message | string | 日志信息 |
| response.data.sender_message_id | string | 消息ID |
| response.data.sender_type | string | 发送器类型 |
| response.data.status | string | 状态 |
| response.total | string | 总数 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/message_logs
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
      "message_id":"55",
       "count_num":true,
        "page":{
            "page":1,
            "limit":10
        }
}
```

```json
{
  "response": {
    "data": [
      {
        "app_id": "16",
        "create_time": "1749875409",
        "executor_type": "",
        "id": "2246",
        "log_type": "2",
        "message": "tpl id [ddddd] not found",
        "sender_message_id": "55",
        "sender_type": "2",
        "status": "3"
      }
    ],
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```