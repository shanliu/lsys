### 查询子应用信息

> payload参数

| 参数 | 类型 | 是否必填 | 描述 |
|------|------|----------|------|
| client_id | string | 是 | 应用KEY |

> 响应参数

| 参数 | 类型 | 描述 |
|------|------|------|
| response.client_id | string | 应用KEY |
| response.name | string | 应用名称 |
| response.user_data.user_data | string | 用户标识 |
| response.user_data.user_id | string | 用户ID |
| response.user_data.user_nickname | string | 用户昵称 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /rest/app?method=sub_app_user
Content-type:application/json

{
    "client_id": "Sapp00122"
}
```

```json
{
  "response": {
    "client_id": "dd9319fss",
    "name": "dd11127",
    "user_data": {
      "app_id": "0",
      "id": "33",
      "user_account": "aaaaaa333",
      "user_data": "7",
      "user_nickname": "x11"
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```