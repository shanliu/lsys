> 示例

```http
POST /api/user/app_rbac/role/user_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "role_id": 1,
    "all":true,
    "count_num":true,
    "page":{
      "page":1,
      "limit":10
   }
}
```