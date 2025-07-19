> 示例

```http
POST /api/user/app_rbac/role/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":1,
     "use_app_user":false,
    "user_param":"xx",
    "user_range": 1,
    "res_range": 1,
    "role_name":"xxx",
    "role_key":"xxx"
}
```