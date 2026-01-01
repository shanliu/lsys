### 删除角色权限

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| role_id | int | 是 | 角色ID |
| perm_data | array | 是 | 权限数据列表 |
| perm_data.op_id | int | 是 | 操作ID |
| perm_data.res_id | int | 是 | 资源ID |

> 响应参数 

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/role/perm_delete
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 11,
     "perm_data":[{
      "op_id":1,
      "res_id":3
     }]
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