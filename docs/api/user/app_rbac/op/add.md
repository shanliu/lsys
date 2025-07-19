### 添加应用操作权限

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| app_id | int | 应用ID |
| use_app_user | boolean | 是否使用应用用户 |
| user_param | string | 用户参数 |
| op_key | string | 操作键名 |
| op_name | string | 操作名称 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 操作权限ID |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_rbac/op/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
    "use_app_user":false,
    "user_param":"xx3",
    "op_key": "xx2",
    "op_name": "xx3"
}
```

```json
{
  "response": {
    "id": "12"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```