### OAuth登录秘钥添加

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| secret | string | 否 | 秘钥 |
| secret_timeout | int | 是 | 秘钥超时时间(秒) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | string | 生成的秘钥值 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/oauth_client_secret_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 1,
   "secret": null,
   "secret_timeout": 10000000000
}
```

```json
{
  "response": {
    "data": "5884fa0c24f11d5bf405edd224e5cb92"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```