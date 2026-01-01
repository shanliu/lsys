

### 获取短信通知配置

> 请求参数

无需传递参数

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].app_id | string | 应用ID |
| response.data[].app_name | string | 应用名称 |
| response.data[].call_url | string | 回调URL |
| response.data[].change_time | string | 修改时间(秒) |
| response.data[].change_user_id | string | 修改人ID |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/notify_get_config
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}

```

```json
{
  "response": {
    "data": [
      {
        "app_id": "16",
        "app_name": "测试aPP",
        "call_url": null,
        "change_time": null,
        "change_user_id": null
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```