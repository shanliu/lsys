### OAuth登录秘钥更改

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| old_secret | string | 是 | 旧秘钥 |
| secret | string | 是 | 新秘钥 |
| secret_timeout | int | 是 | 秘钥超时时间 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | string | 新的秘钥值 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/oauth_client_secret_change
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 1,
   "old_secret":"999999999999999999999999",
   "secret": "999999999999999999999999",
   "secret_timeout": 0
}
```


```json
{
  "response": {
    "data": "999999999999999999999999"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```