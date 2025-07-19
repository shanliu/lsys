### 添加角色

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| user_range | int | 用户范围 1:指定用户 |
| res_range | int | 资源范围 1:包含指定授权 2:访问任意资源 3:禁止指定授权 |
| role_name | string | 角色名称 |
| role_key | string | 角色key |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 角色ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |


> 示例

```http
POST /rest/rbac/role?method=add
Content-type:application/json

{
  "use_app_user":false,
     "user_param": "account_11",
   "user_range": 1,
    "res_range": 3,
    "role_name":"xxxp2",
    "role_key":""
}
```

```json
{
  "response": {
    "id": "13"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```