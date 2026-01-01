> 示例

```http
POST /api/user/app_rbac/role/perm_delete
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 1,
     "perm_data":[{
      "op_id":1,
      "res_id":1
     }]
}
```