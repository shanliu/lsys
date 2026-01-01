### 添加应用密钥

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| secret | string | 否 | 密钥值 |
| secret_timeout | int | 是 | 密钥超时时间(秒) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | string | 生成的密钥值 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/app_secret_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 1,
   "secret": null,
   "secret_timeout": 8
}
```

```json
{
  "response": {
    "data": "3cd3318febfa340508a333be72bdd87d"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```