> 示例

```http
POST /api/user/app/request_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "id":null,
   "app_id":5,
   "request_type":1,
   "status":null,
   "count_num":true,
   "page":{
      "page":1,
      "limit":10
   }
}
```


```
{
  "response": {
    "data": [
      {
        "app_client": "dd9319fss",
        "app_id": "5",
        "app_name": "dd11127",
        "app_status": "2",
        "change_data": {
          "client_id": "dd9319fss",
          "name": "dd11127"
        },
        "confirm_note": "ssss",
        "confirm_time": "1748417249",
        "confirm_user_data": {
          "app_id": "0",
          "id": "1",
          "user_account": "a**a",
          "user_data": "1",
          "user_nickname": "r**t"
        },
        "confirm_user_id": "1",
        "feature_data": null,
        "id": "7",
        "oauth_client_data": null,
        "parent_app_client_id": "app001",
        "parent_app_id": "1",
        "parent_app_name": "测试应用",
        "request_time": "1748417220",
        "request_type": "1",
        "request_user_id": "1",
        "status": "2"
      }
    ],
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```