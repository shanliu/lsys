### 容联云短信配置列表接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | 配置ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| response.data[].account_sid | string | 账户SID |
| response.data[].account_token | string | 账户令牌 |
| response.data[].hide_account_sid | string | 隐藏后的账户SID |
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
POST /api/system/sender/smser/cloopen_config_list
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
        "account_sid": "ssssffff",
        "account_token": "ccc333c",
        "callback_key": "1313e12rddd",
        "callback_url": "http://127.0.0.1:8080/notify/sms/9/1313e12rddd",
        "change_time": "1749876482",
        "change_user_id": "7",
        "hide_account_sid": "ss**ff",
        "id": "9",
        "limit": "11",
        "name": "bbbbddddd",
        "sms_app_id": "111"
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