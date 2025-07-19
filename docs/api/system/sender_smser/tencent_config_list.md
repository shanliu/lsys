### 腾讯云短信配置列表接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | 配置ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| response.data[].region | string | 区域 |
| response.data[].secret_id | string | 密钥ID |
| response.data[].secret_key | string | 密钥Key |
| response.data[].hide_secret_id | string | 隐藏后的密钥ID |
| response.data[].sms_app_id | string | 短信应用ID |
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
POST /api/system/sender/smser/tencent_config_list
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
        "callback_key": "cccccccc",
        "callback_url": "http://127.0.0.1:8080/notify/sms/17/cccccccc",
        "change_time": "1748006921",
        "change_user_id": "1",
        "hide_secret_id": "cc**cc",
        "id": "17",
        "limit": "11",
        "name": "bbbb",
        "region": "xx",
        "secret_id": "cccccccccccccccc",
        "secret_key": "cccccccccccccccc",
        "sms_app_id": "cccccccccccc"
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