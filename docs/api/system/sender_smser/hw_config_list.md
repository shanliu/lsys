### 华为云短信配置列表接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | 配置ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| response.data[].url | string | 接口地址 |
| response.data[].app_key | string | 应用Key |
| response.data[].app_secret | string | 应用密钥 |
| response.data[].hide_app_key | string | 隐藏后的应用Key |
| response.data[].callback_key | string | 回调密钥 |
| response.data[].callback_url | string | 回调URL |
| response.data[].limit | string | 限制数量 |
| response.data[].change_time | string | 修改时间 |
| response.data[].change_user_id | string | 修改用户ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http

POST /api/system/sender/smser/hw_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "ids":null
}
```

```json
{
  "response": {
    "data": [
      {
        "app_key": "dddddddddddddd",
        "app_secret": "dddddddddddddd",
        "callback_key": "ddddddd",
        "callback_url": "http://127.0.0.1:8080/notify/sms/10/ddddddd",
        "change_time": "1748006547",
        "change_user_id": "1",
        "hide_app_key": "dd**dd",
        "id": "10",
        "limit": "11",
        "name": "bbbb",
        "url": "http://rage1.xxx.com/dddd"
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