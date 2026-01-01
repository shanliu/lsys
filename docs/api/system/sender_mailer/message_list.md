
### 邮件消息列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| tpl_key | string | 否 | 模板键值 |
| status | int | 否 | 状态 |
| body_id | int | 否 | 消息体ID |
| snid | string | 否 | 消息序列号 |
| to_mail | string | 否 | 接收邮箱 |
| count_num | boolean | 否 | 是否统计总数 |
| limit.pos | int | 是 | 起始位置 |
| limit.limit | int | 是 | 每页数量 |
| limit.forward | boolean | 是 | 是否向前查询 |
| limit.more | boolean | 是 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 消息列表 |
| response.data.add_time | int | 添加时间（秒） |
| response.data.app_id | string | 应用ID |
| response.data.expected_time | int | 预期发送时间（秒） |
| response.data.id | string | 消息ID |
| response.data.max_try_num | string | 最大重试次数 |
| response.data.now_send | string | 当前是否发送 |
| response.data.on_task | string | 是否在任务中 |
| response.data.send_time | int | 发送时间（秒） |
| response.data.snid | string | 消息序列号 |
| response.data.status | string | 状态码 |
| response.data.to_mail | string | 接收邮箱 |
| response.data.tpl_key | string | 模板键值 |
| response.data.try_num | string | 已重试次数 |
| response.next | string | 下一页标记 |
| response.total | string | 总数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/message_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
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
        "add_time": "1749217998",
        "app_id": "0",
        "expected_time": "1749217998",
        "id": "49",
        "max_try_num": "1",
        "now_send": "0",
        "on_task": "0",
        "send_time": "1749218000",
        "snid": "7336752049343909888",
        "status": "5",
        "to_mail": "rustlang@qq.com",
        "tpl_key": "valid_code_login_email",
        "try_num": "1"
      }
    ],
    "next": null,
    "total": "10"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
