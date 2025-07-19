> 示例

```http
POST /api/user/app_rbac/res/type_op_del
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":1,
    "use_app_user":false,
   "user_param":"xx",
   "res_type":"11",
   "op_ids":[1]
}
```