> 示例


```http
POST /api/user/app_rbac/role/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":1,
     "use_app_user":false,
    "user_param":"xx",
    "role_name":"xxx",
    "role_key":"xxx",
    "user_data":1,
    "res_count":true,
    "res_op_count":true,
    "user_count":true,
    "ids":[1],
    "user_range": 1,
    "res_range": 1,
     "count_num":true,
    "page":{
      "page":1,
      "limit":10
   }
}
```