
### 获取短信发送日志

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | string | 是 | 短信消息ID |
| count_num | boolean | 是 | 是否统计总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].app_id | string | 应用ID |
| response.data[].create_time | string | 创建时间(秒) |
| response.data[].executor_type | string | 执行器类型 |
| response.data[].id | string | 日志ID |
| response.data[].log_type | string | 日志类型 |
| response.data[].message | string | 日志消息 |
| response.data[].sender_message_id | string | 短信消息ID |
| response.data[].sender_type | string | 发送器类型 |
| response.data[].status | string | 状态 |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/message_logs
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
      "message_id":"1",
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
        "app_id": "0",
        "create_time": "1748006030",
        "executor_type": "",
        "id": "1",
        "log_type": "2",
        "message": "tpl id [valid_code_register_mobile] not found",
        "sender_message_id": "1",
        "sender_type": "1",
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
