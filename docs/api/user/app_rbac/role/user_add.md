> 示例

```http
POST /api/user/app_rbac/role/user_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 1,
      "user_data":[
        {
          "use_app_user":false,
          "user_param":"xxxx",
          "timeout":0
        }
      ]
}
```