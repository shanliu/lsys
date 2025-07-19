
### 发送短信

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mobile | array | 是 | 手机号码列表 |
| app_id | int | 是 | 应用ID |
| tpl_key | string | 是 | 模板标识 |
| data | object | 是 | 模板数据 |
| send_time | string | 否 | 定时发送时间 |
| max_try | int | 否 | 最大重试次数 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/message_send
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "mobile":["13800138000"],
    "app_id":16,
    "tpl_key": "ddddd",
    "data":{"code":"11","aa":"111"},
    "send_time":"2024-12-11 10:00:00",
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
