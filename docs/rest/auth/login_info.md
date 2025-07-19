### 获取登录信息

> payload参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| token_data | string | 是 | 登录令牌 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.session.add_time | int | 添加时间 |
| response.session.device_id | string | 设备ID |
| response.session.device_name | string | 设备名称 |
| response.session.expire_time | int | 过期时间 |
| response.session.id | string | 会话ID |
| response.session.login_ip | string | 登录IP |
| response.session.login_type | string | 登录类型 |
| response.session.logout_time | int | 登出时间 |
| response.session.oauth_app_id | string | OAuth应用ID |
| response.session.source_token_data | string | 源token数据 |
| response.session.status | string | 状态 |
| response.session.token_data | string | token数据 |
| response.session.user_app_id | string | 用户应用ID |
| response.session.user_id | string | 用户ID |
| response.user.app_id | string | 应用ID |
| response.user.change_time | int | 修改时间 |
| response.user.id | string | 用户ID |
| response.user.user_account | string | 用户账号 |
| response.user.user_data | string | 用户数据 |
| response.user.user_nickname | string | 用户昵称 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /rest/auth?method=login_info
Content-type:application/json

{
   "token_data": "926896f3a7972a2427d5002b061d5700"
}
```

```json
{
  "response": {
    "session": {
      "add_time": "1749454557",
      "device_id": "xx",
      "device_name": "xx",
      "expire_time": "1749464557",
      "id": "12",
      "login_ip": "127.0.0.1",
      "login_type": "code",
      "logout_time": "0",
      "oauth_app_id": "0",
      "source_token_data": "",
      "status": "1",
      "token_data": "926896f3a7972a2427d5002b061d5700",
      "user_app_id": "1",
      "user_id": "17"
    },
    "user": {
      "app_id": "1",
      "change_time": "1749454557",
      "id": "17",
      "user_account": "xx",
      "user_data": "xx",
      "user_nickname": "xx"
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```