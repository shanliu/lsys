> 示例

```http
POST /api/user/app/request_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":5,
   "status":null,
   "count_num":true,
   "page":{
      "page":1,
      "limit":10
   }
}
```