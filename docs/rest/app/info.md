### 查询子应用信息

> payload参数

| 参数 | 类型 | 是否必填 | 描述 |
|------|------|----------|------|
| client_id | string | 是 | 应用KEY |
| user_data | bool | 是 | 是否返回应用对应的用户信息 |

> 响应参数

| 参数 | 类型 | 描述 |
|------|------|------|
| response.client_id | string | 应用KEY |
| response.name | string | 应用名称 |
| response.sub_secret.secret_data | string | 密钥数据 |
| response.sub_secret.time_out | int | 超时时间 |
| response.user_data.user_data | string | 用户标识 |
| response.user_data.user_id | string | 用户ID |
| response.user_data.user_nickname | string | 用户昵称 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /rest/app?method=info
Content-type:application/json

{
    "client_id": "dd9319fss",
    "user_data":true
}
```

```json
{
  "response": {
    "client_id": "dd9319fss",
    "name": "dd11127",
    "sub_secret": [
      {
        "secret_data": "482a5edc4b943eb9796ed7492d3a1df3",
        "time_out": "0"
      }
    ],
    "user_data": {
      "user_data": "1",
      "user_id": "1",
      "user_nickname": "root"
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```