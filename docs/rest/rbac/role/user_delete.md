### 删除角色用户

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| role_id | int | 角色ID |
| user_data | array | 用户ID数组 |


> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |


> 示例

```http
POST /rest/rbac/role?method=user_delete
Content-type:application/json

{
  "use_app_user":false,
     "user_param": "account_11",
     "role_id": 13,
     "user_data":[
       51
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

