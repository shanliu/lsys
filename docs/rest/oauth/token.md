### 获取Token接口

> 请求参数

| 参数名         | 类型   | 必填 | 说明               |
| -------------- | ------ | ---- | ------------------ |
| client_id      | string | 是   | 应用的Client ID    |
| client_secret  | string | 是   | 应用的Client Secret|
| code           | string | 是   | 授权码             |

> 响应参数

| 参数名                        | 类型      | 说明                   |
| ----------------------------- | --------- | ---------------------- |
| response.access_token         | string    | 访问令牌               |
| response.expires_in           | int       | 令牌有效期(秒)         |
| response.openid               | string    | 用户唯一标识           |
| response.refresh_token        | string    | 刷新令牌 |
| response.scope                | array     | 授权范围               |
| result.code                   | string    | 结果码                 |
| result.message                | string    | 结果信息               |
| result.state                  | string    | 状态                   |


> 示例

```http
# 跳转到登录页面
GET http://www.lsys.cc/oauth.html?client_id={{APP_CLIENT_ID}}&redirect_uri=http%3A%2F%2F127.0.0.1%3A8080%2F&response_type=code&scope=user_info,mail,mobile&state=aa
# 登录页使用相关接口参见:
#  `/public/oauth/login.md`
#  `/public/oauth/scope.md`
```

```http
GET /oauth/token?client_id={{APP_CLIENT_ID}}&client_secret={{APP_OAUTH_SECRET}}&code=8d6fcebf70b3264ca8f42a6ee369b285
```

```json
{
  "response": {
    "access_token": "walaskpjahbnaozixztmrbobudiqszxx",
    "expires_in": "1750086205",
    "openid": "1",
    "refresh_token": null,
    "scope": [
      "user_address"
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
