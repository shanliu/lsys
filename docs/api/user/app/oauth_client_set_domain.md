### 设置OAuth登录回调域名

> 请求参数

| 参数名 | 类型 | 是否必须 | 说明 |
|--------|------|----------|------|
| app_id | int | 是 | 应用ID |
| callback_domain | string | 是 | 回调域名 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/user/app/oauth_client_set_domain
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "app_id": 15,
     "callback_domain":"xxx.com"
}
```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```