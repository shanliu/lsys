
### 添加角色
> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_range | int | 是 | 用户范围 |
| res_range | int | 是 | 资源范围 |
| role_name | string | 是 | 角色名称 |
| role_key | string | 否 | 角色标识 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 新增角色ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/rbac/role/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "user_range": 1,
    "res_range": 1,
    "role_name":"xxx1",
    "role_key":""
}
```

```json
{
  "response": {
    "id": "21"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
