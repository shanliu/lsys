> 示例

```http
POST /api/user/app_rbac/res/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":1,
    "use_app_user":false,
    "user_param":"xx",
    "res_type":"111",
    "res_data":"111",
    "res_name": "111",
    "ids":[11],
    "count_num":true,
    "page":{
      "page":1,
      "limit":10
   }
}
```