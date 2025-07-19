

> 示例

```http
POST /api/user/app_rbac/res/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":1,
    "use_app_user":false,
   "user_param":"xx",
   "res_name":"11",
   "res_type":"11",
   "res_data":"11"
}
```