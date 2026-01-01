### 网易云短信配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| access_key | string | 是 | 访问密钥 |
| access_secret | string | 是 | 访问密钥密文 |
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
POST /api/system/sender/smser/netease_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "bccccbbb",
    "access_key":"cccccccccccc",
    "access_secret":"cccccccccccc",
    "limit":11
}
```

```json
{
  "response": {
    "id": "26"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```