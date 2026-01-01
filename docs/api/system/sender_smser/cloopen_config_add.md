### 容联云短信配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| account_sid | string | 是 | 账户SID |
| account_token | string | 是 | 账户令牌 |
| sms_app_id | string | 是 | 短信应用ID |
| limit | int | 否 | 限制数量 |
| callback_key | string | 否 | 回调密钥 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

###  cloopen短信配置添加

> 示例

```http
POST /api/system/sender/smser/cloopen_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "bbbbe2",
    "account_sid": "ssss",
    "account_token":"cccc",
    "sms_app_id":"111",
    "limit":11,
    "callback_key":"ddddddddddd"
}
```

```json
{
  "response": {
    "id": "24"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```