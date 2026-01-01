### 腾讯云短信配置编辑接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 是 | 配置ID |
| name | string | 是 | 配置名称 |
| region | string | 是 | 区域 |
| secret_id | string | 是 | 密钥ID |
| secret_key | string | 是 | 密钥Key |
| sms_app_id | string | 是 | 短信应用ID |
| callback_key | string | 否 | 回调密钥 |
| limit | int | 否 | 限制数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.num | string | 更新数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/tencent_config_edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id": 18,
   "name": "bb121212bb",
    "region":"xx",
    "secret_id":"cccc21212",
    "secret_key":"cc2121cc",
    "secret_id":"cccc21212",
    "sms_app_id":"cc2121c2121ccc",
     "callback_key":"cc2121ccc",
    "limit":11
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