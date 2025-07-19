> 示例

```http
POST /api/user/app_rbac/role/available_user
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":1,
   "user_any":null,
   "limit":{
        "pos":0,
        "limit":10,
        "forward":true,
       "more":true
    }
}

```