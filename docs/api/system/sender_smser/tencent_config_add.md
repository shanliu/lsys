### 腾讯云短信配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
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
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/tencent_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "b俄22bbb",
    "region":"xx",
    "secret_id":"cccccccccccccccc",
    "secret_key":"cccccccccccccccc",
    "sms_app_id":"cccccccccccc",
     "callback_key":"cccccccc",
    "limit":11
}
```

```json
{
  "response": {
    "id": "25"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```