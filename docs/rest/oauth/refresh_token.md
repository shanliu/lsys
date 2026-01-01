### 刷新访问令牌

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| client_id | string | 是 | 应用客户端ID |
| client_secret | string | 是 | 应用密钥 |
| refresh_token | string | 是 | 刷新令牌 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.access_token | string | 访问令牌 |
| response.expires_in | int | 过期时间(秒) |
| response.openid | string | 用户开放ID |
| response.refresh_token | string | 刷新令牌 |
| response.scope | array | 权限范围 |
| result.code | string | 状态码 |
| result.message | string | 状态消息 |
| result.state | string | 状态标识 |


> 示例

```http
GET /oauth/refresh_token?client_id={{APP_CLIENT_ID}}&client_secret={{APP_OAUTH_SECRET}}&refresh_token=imilpgoozbfimtaxhvsvhbnnyaqjjghb

```

```json
{
  "response": {
    "access_token": "azazajnwphflivdhnkazezbnsreguxlv",
    "expires_in": "1750092736",
    "openid": "1",
    "refresh_token": "lmihyglqhdsuncjuhprruhkqmqfkraqn",
    "scope": []
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```


