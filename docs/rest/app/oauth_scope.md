### 查询子应用OAauth登录已申请的SCOPE

> payload参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| client_id | string | 是 | 客户端ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.client_id | string | 客户端ID |
| response.name | string | 应用名称 |
| response.scope_data | array | 权限范围数据 |
| response.scope_data.desc | string | 权限描述 |
| response.scope_data.key | string | 权限标识 |
| response.scope_data.name | string | 权限名称 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /rest/app?method=sub_app_oauth_scope
Content-type:application/json

{
    "client_id": "dd9319fss"
}
```

```json
{
  "response": {
    "client_id": "dd9319fss",
    "name": "dd11127",
    "scope_data": [
      {
        "desc": "获取用户的邮箱地址",
        "key": "mail",
        "name": "邮箱"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```