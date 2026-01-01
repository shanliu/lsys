### 短信消息列表接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| tpl_key | string | 否 | 模板键名筛选 |
| status | int | 否 | 状态筛选 |
| body_id | string | 否 | 消息体ID筛选 |
| snid | string | 否 | 消息唯一标识筛选 |
| mobile | string | 否 | 手机号筛选 |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | int | 否 | 起始位置 |
| limit.limit | int | 否 | 每页数量 |
| limit.forward | boolean | 否 | 是否向前查询 |
| limit.more | boolean | 否 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 消息ID |
| response.data[].app_id | string | 应用ID |
| response.data[].mobile | string | 手机号 |
| response.data[].snid | string | 消息唯一标识 |
| response.data[].tpl_key | string | 模板键名 |
| response.data[].status | string | 状态 |
| response.data[].add_time | string | 添加时间 |
| response.data[].expected_time | string | 预期发送时间 |
| response.data[].send_time | string | 实际发送时间 |
| response.data[].max_try_num | string | 最大尝试次数 |
| response.data[].try_num | string | 已尝试次数 |
| response.data[].now_send | string | 是否正在发送 |
| response.data[].on_task | string | 是否在任务中 |
| response.next | string | 下一页标识 |
| response.total | string | 总记录数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/message_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "tpl_key":null,
   "status":null,
   "body_id":null,
   "snid":null,
   "mobile":null,
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
        "add_time": "1749645902",
        "app_id": "0",
        "expected_time": "1749645902",
        "id": "8",
        "max_try_num": "1",
        "mobile": "86-13800138001",
        "now_send": "0",
        "on_task": "0",
        "send_time": "0",
        "snid": "7338546809096327168",
        "status": "3",
        "tpl_key": "valid_code_login_mobile",
        "try_num": "1"
      }
    ],
    "next": null,
    "total": "8"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```