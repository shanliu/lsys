### 获取邮件发送消息列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| tpl_id | int | 否 | 模板ID |
| status | int | 否 | 状态 |
| body_id | int | 否 | 内容ID |
| snid | string | 否 | 消息序列号 |
| to_mail | string | 否 | 收件人邮箱 |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | int | 是 | 起始位置 |
| limit.limit | int | 是 | 获取数量 |
| limit.forward | boolean | 是 | 是否向前获取 |
| limit.more | boolean | 是 | 是否获取更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.add_time | string | 添加时间(秒) |
| response.data.app_id | string | 应用ID |
| response.data.expected_time | string | 预期发送时间(秒) |
| response.data.id | string | 消息ID |
| response.data.max_try_num | string | 最大重试次数 |
| response.data.now_send | string | 正在发送标记 |
| response.data.on_task | string | 任务状态 |
| response.data.send_time | string | 发送时间(秒) |
| response.data.snid | string | 消息序列号 |
| response.data.status | string | 状态 |
| response.data.to_mail | string | 收件人邮箱 |
| response.data.tpl_key | string | 模板标识 |
| response.data.try_num | string | 已重试次数 |
| response.next | string | 下一页标记 |
| response.total | string | 总数 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |

> 示例

```http
POST /api/user/app_sender/mailer/message_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":1,
    "tpl_id":null,
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
        "add_time": "1749875409",
        "app_id": "16",
        "expected_time": "1749875409",
        "id": "55",
        "max_try_num": "1",
        "now_send": "0",
        "on_task": "0",
        "send_time": "0",
        "snid": "7339509429441875968",
        "status": "3",
        "to_mail": "rustlang@qq.com",
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