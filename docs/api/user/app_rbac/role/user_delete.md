> 示例

```http
POST /api/user/app_rbac/role/user_delete
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 1,
     "use_app_user":false,
     "user_data":[
       "xxxx"
      ]
}
```