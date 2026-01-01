### 账号名注册

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| nikename | string | 是 | 昵称 |
| name | string | 是 | 用户名 |
| password | string | 是 | 密码 |

> 响应参数 

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 用户ID |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 | 
| result.state | string | 响应状态 |


> 示例

```http
POST /api/auth/register/name
Content-Type: application/json

{
    "nikename":"x11",
    "name": "aaaaaff",
    "password": "qq001200"
}
```

```json
{
  "response": {
    "id": "2"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```