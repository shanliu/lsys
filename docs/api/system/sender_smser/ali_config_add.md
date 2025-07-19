### 阿里云短信配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| access_id | string | 是 | 访问ID |
| access_secret | string | 是 | 访问密钥 |
| region | string | 是 | 区域 |
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

POST /api/system/sender/smser/ali_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "bbbb33322",
    "access_id": "ssss",
    "access_secret":"cccc",
    "region":"111",
    "callback_key":"xxx111111",
    "limit":11
}
```

```json
{
  "response": {
    "id": "23"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```