### 指定用户退出登录

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| oauth_app_id | int | 是 | OAuth应用ID |
| token_data | string | 是 | 用户登录令牌，可从登录历史获取 `/api/system/user/login_history`  |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/system/user/user_logout
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":0,
    "oauth_app_id":0,
    "token_data":"WGWILJDOQIEQNVRZUWCGJTARKMGQHNBK"
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

