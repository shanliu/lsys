
### 删除角色用户

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| role_id | int | 角色ID |
| user_data | []int | 用户ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/role/user_delete
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 12,
     "user_data":[
       4
      ]
}
```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```