### 容联云短信配置编辑接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 是 | 配置ID |
| name | string | 是 | 配置名称 |
| account_sid | string | 是 | 账户SID |
| account_token | string | 是 | 账户令牌 |
| sms_app_id | string | 是 | 短信应用ID |
| limit | int | 否 | 限制数量 |
| callback_key | string | 否 | 回调密钥 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.num | string | 更新数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/cloopen_config_edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id": 9,
   "name": "bbbbddddd",
    "account_sid": "ssssffff",
    "account_token":"ccc333c",
    "sms_app_id":"111",
    "limit":11,
    "callback_key":"1313e12rddd"
}
```

```json
{
  "response": {
    "num": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```