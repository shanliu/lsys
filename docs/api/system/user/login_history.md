### 查看指定应用的登录历史

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID,0为系统 |
| oauth_app_id | int | 否 | OAuth应用ID,0为非OAUTH登录 |
| user_id | int | 否 | 用户ID |
| is_enable | int | 否 | 是否登录 |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | int | 否 | 起始位置 |
| limit.limit | int | 否 | 每页数量 |
| limit.forward | boolean | 否 | 是否向前查询 |
| limit.more | boolean | 否 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.add_time | int | 添加时间 |
| response.data.app_id | int | 应用ID |
| response.data.device_id | string | 设备ID |
| response.data.device_name | string | 设备名称 |
| response.data.expire_time | int | 过期时间 |
| response.data.login_ip | string | 登录IP |
| response.data.login_type | string | 登录类型 |
| response.data.logout_time | int | 登出时间 |
| response.data.oauth_app_id | int | OAuth应用ID |
| response.data.status | int | 状态 |
| response.data.token_data | string | 令牌数据 |
| response.data.user_id | int | 用户ID |
| response.next | string | 下一页标识 |
| response.total | int | 总数量 |
| result.code | string | 返回码 |
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/system/user/login_history
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":0,
    "oauth_app_id":null,
    "user_id":null,
    "is_enable":null,
    "count_num":true,
    "limit":{
        "pos":0,
        "limit":10,
        "forward":true,
        "more":true
    }
}

```

```json
{
  "response": {
    "data": [
      {
        "add_time": "1749440030",
        "app_id": "0",
        "device_id": "",
        "device_name": "",
        "expire_time": "1749699230",
        "login_ip": "127.0.0.1",
        "login_type": "name",
        "logout_time": "0",
        "oauth_app_id": "0",
        "status": "1",
        "token_data": "WGWILJDOQIEQNVRZUWCGJTARKMGQHNBK",
        "user_id": "1"
      }
    ],
    "next": null,
    "total": "9"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```