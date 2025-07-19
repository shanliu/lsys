### 系统变更日志

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| log_type | string | 否 | 日志类型 |
| add_user_id | int | 否 | 添加用户ID |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | int | 是 | 分页起始位置 |
| limit.limit | int | 是 | 每页数量 |
| limit.forward | boolean | 是 | 是否向前查询 |
| limit.more | boolean | 是 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.add_time | int | 添加时间 |
| response.data.add_user_id | string | 添加用户ID |
| response.data.add_user_ip | string | 添加用户IP |
| response.data.device_id | string | 设备ID |
| response.data.id | string | 记录ID |
| response.data.log_data | string | 日志详细数据 |
| response.data.log_type | string | 日志类型 |
| response.data.message | string | 日志消息 |
| response.data.request_id | string | 请求ID |
| response.data.request_user_agent | string | 用户代理 |
| response.data.source_id | string | 来源ID |
| response.next | string | 下一页标识 |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态标识 |

> 示例

```http
POST /api/system/user/change_logs
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "log_type":null,
    "add_user_id":null,
    "count_num":true,
    "limit":{
        "pos":3,
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
        "add_time": "1748404339",
        "add_user_id": "1",
        "add_user_ip": "127.0.0.1",
        "device_id": "",
        "id": "13",
        "log_data": "{\"method\":\"sub_app_notify\",\"url\":\"http://127.0.0.1/aa.php\",\"user_id\":1}",
        "log_type": "app-notify-set",
        "message": "set sub_app_notify notify url",
        "request_id": "ue0x4as9u0la7rpc",
        "request_user_agent": "httpyac",
        "source_id": "1"
      }
    ],
    "next": "1",
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
