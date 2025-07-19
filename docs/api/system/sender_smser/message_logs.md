### 短信发送日志查询接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| message_id | int | 是 | 消息ID |
| count_num | string | 否 | 是否返回总数 |
| page.page | int | 否 | 页码 |
| page.limit | int | 否 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 日志ID |
| response.data[].app_id | string | 应用ID |
| response.data[].sender_message_id | string | 发送消息ID |
| response.data[].sender_type | string | 发送器类型 |
| response.data[].executor_type | string | 执行器类型 |
| response.data[].log_type | string | 日志类型 |
| response.data[].status | string | 状态 |
| response.data[].message | string | 日志消息 |
| response.data[].create_time | string | 创建时间 |
| response.total | string | 总记录数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/message_logs
Content-Type : application/json
Authorization : Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "message_id":2,
   "count_num":"1",
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
        "create_time": "1748270209",
        "executor_type": "ali-sms-config",
        "id": "2209",
        "log_type": "2",
        "message": "api fail,msg:Specified access key is not found.",
        "sender_message_id": "2",
        "sender_type": "1",
        "status": "3"
      }
    ],
    "total": "452"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```