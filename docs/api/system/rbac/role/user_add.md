
### 添加角色用户

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| role_id | int | 角色ID |
| user_data | array | 用户数据列表 |
| user_data.user_id | int | 用户ID |
| user_data.timeout | int | 超时时间(秒) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |



> 示例

```http
POST /api/system/rbac/role/user_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 12,
      "user_data":[
       { 
        "user_id":7,
        "timeout":0
        }
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
