### 系统应用登录接口

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| token_code | string | 令牌码,生成16-64位字符 |
| user_data | string | 用户数据 |
| user_nickname | string | 用户昵称 |
| expire_time | int | 过期时间 |
| device_name | string | 设备名称 |
| user_account | string | 用户账号 |
| login_ip | string | 登录IP |
| device_id | string | 设备ID |
| session_data | object | 会话数据 |
| session_data.s1 | string | 会话数据值 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.token_data | string | 令牌数据,用户获取登录信息或完成APPCODE登录 |
| response.user_id | string | 用户ID |
| response.user_nickname | string | 用户昵称 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /rest/auth?method=do_login
Content-type:application/json

{
   "token_code":"11112121211121212111d11",
   "user_data":"xx",
   "user_nickname":"xx",
   "expire_time":10000,
   "device_name":"xx",
   "user_account":"xx",
   "login_ip":"127.0.0.1",
   "device_id":"xx",
   "session_data":{
        "s1":"v1"
   }
}
```

```json
{
  "response": {
    "token_data": "926896f3a7972a2427d5002b061d5700",
    "user_id": "17",
    "user_nickname": "xx"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```