### 添加角色权限

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| role_id | int | 角色ID |
| perm_data | array | 权限数据数组 |
| perm_data[].op_id | int | 操作ID |
| perm_data[].res_id | int | 资源ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |


> 示例

```http
POST /rest/rbac/role?method=perm_add
Content-type:application/json

{
   "use_app_user":false,
     "user_param": "account_11",
  "role_id": 20,
     "perm_data":[{
      "op_id":11,
      "res_id":9
     }]
}
```

