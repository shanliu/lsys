### 应用回调通知列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | string | 否 | 应用ID |
| method | string | 否 | 通知方法 |
| status | string | 否 | 通知状态 |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | int | 是 | 起始位置 |
| limit.limit | int | 是 | 每页数量 |
| limit.forward | boolean | 是 | 是否向前查询 |
| limit.more | boolean | 是 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.app_id | string | 应用ID |
| response.data.call_url | string | 回调URL |
| response.data.id | string | 通知ID |
| response.data.next_time | int | 下次通知时间 |
| response.data.notify_key | string | 通知键值 |
| response.data.notify_method | string | 通知方法 |
| response.data.notify_type | string | 通知类型 |
| response.data.publish_time | int | 发布时间 |
| response.data.result | string | 通知结果 |
| response.data.status | string | 通知状态 |
| response.data.try_max | int | 最大重试次数 |
| response.data.try_num | int | 当前重试次数 |
| response.next | string | 下一页标识 |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态标识 |

> 示例

```http
POST /api/user/app/notify_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id": null,
    "method": null,
    "status":null,
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
        "app_id": "1",
        "call_url": "https://www.baidu.com/",
        "id": "9",
        "next_time": "1748596414",
        "notify_key": "5",
        "notify_method": "sub_app_notify",
        "notify_type": "1",
        "publish_time": "1748596354",
        "result": "Code:500 Res:{\"status\":\"success\",\"message\":\"Data saved\"}",
        "status": "3",
        "try_max": "2",
        "try_num": "2"
      }
    ],
    "next": null,
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```