### 华为云短信配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| url | string | 是 | 接口地址 |
| app_key | string | 是 | 应用Key |
| app_secret | string | 是 | 应用密钥 |
| callback_key | string | 否 | 回调密钥 |
| limit | int | 否 | 限制数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

###  huawei短信配置添加


> 示例

```http
POST /api/system/sender/smser/hw_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "bbbcb",
    "url": "http://rage1.xxx.com",
    "app_key":"cccccccccccc",
    "app_secret":"cccccccccccccccc",
    "callback_key":"11cccc1",
    "limit":11
}
```

```json
{
  "response": {
    "id": "28"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```