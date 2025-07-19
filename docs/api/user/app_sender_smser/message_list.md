
### 获取短信发送列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| tpl_key | string | 否 | 模板标识 |
| status | int | 否 | 发送状态 |
| body_id | string | 否 | 消息体ID |
| snid | string | 否 | 消息序列号 |
| to_mail | string | 否 | 目标手机号 |
| count_num | boolean | 是 | 是否统计总数 |
| limit.pos | int | 是 | 起始位置 |
| limit.limit | int | 是 | 获取数量 |
| limit.forward | boolean | 是 | 是否向前获取 |
| limit.more | boolean | 是 | 是否获取更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].add_time | string | 添加时间(秒) |
| response.data[].app_id | string | 应用ID |
| response.data[].expected_time | string | 预期发送时间(秒) |
| response.data[].id | string | 消息ID |
| response.data[].max_try_num | string | 最大重试次数 |
| response.data[].mobile | string | 手机号码 |
| response.data[].now_send | string | 当前是否正在发送 |
| response.data[].on_task | string | 是否在任务中 |
| response.data[].send_time | string | 发送时间(秒) |
| response.data[].snid | string | 消息序列号 |
| response.data[].status | string | 发送状态 |
| response.data[].tpl_key | string | 模板标识 |
| response.data[].try_num | string | 已重试次数 |
| response.next | string | 下一页标识 |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |

> 示例

```http
POST /api/user/app_sender/smser/message_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
    "tpl_key":null,
   "status":null,
   "body_id":null,
   "snid":null,
   "to_mail":null,
   "count_num":true,
   "limit":{
        "pos":0,
        "limit":10,
        "forward":true,
        "more":true
    }
}
```

```json
{
  "response": {
    "data": [
      {
        "add_time": "1749877498",
        "app_id": "16",
        "expected_time": "1749877498",
        "id": "10",
        "max_try_num": "1",
        "mobile": "86-13800138000",
        "now_send": "0",
        "on_task": "0",
        "send_time": "0",
        "snid": "7339518192211152896",
        "status": "3",
        "tpl_key": "ddddd",
        "try_num": "1"
      }
    ],
    "next": null,
    "total": "2"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
