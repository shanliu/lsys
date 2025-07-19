### 添加角色

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| user_range | int | 用户范围 |
| res_range | int | 资源范围 |
| role_name | string | 角色名称 |
| role_key | string | 角色标识 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 角色ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/role/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "user_range": 2,
    "res_range": 2,
    "role_name":"xxx1any-2",
    "role_key":"ssss1s"
}
```

```json
{
  "response": {
    "id": "8"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```