### 系统账号搜索


> 示例

```http
POST /api/system/user/account_search
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "key_word":"13800",
    "enable":true,
    "base":true,
    "name":true,
    "info":true,
    "address":true,
    "external":["wechat"],
    "email":[1,2],
    "mobile":[1,2],
    "count_num":true
}
```

```json
{
  "response": {
    "data": [
      {
        "address": [
          {
            "account_id": "4",
            "address_code": "441301",
            "address_detail": "布吉xxx",
            "address_info": "广东深圳龙岗",
            "change_time": "1749898798",
            "country_code": "CHN",
            "id": "6",
            "mobile": "13500135000",
            "name": "xxx",
            "status": "1"
          }
        ],
        "cat": [
          {
            "type": "mobile",
            "val": "13800138001"
          }
        ],
        "email": [
          {
            "account_id": "4",
            "change_time": "1749866683",
            "confirm_time": "0",
            "email": "ssss11121@qq.com",
            "id": "3",
            "status": "1"
          }
        ],
        "external": null,
        "info": {
          "account_id": "4",
          "birthday": "",
          "change_time": "1748007106",
          "gender": "0",
          "headimg": "",
          "id": "2",
          "reg_from": "",
          "reg_ip": "127.0.0.1"
        },
        "mobile": [
          {
            "account_id": "4",
            "area_code": "86",
            "change_time": "1749866697",
            "confirm_time": "0",
            "id": "4",
            "mobile": "13800138005",
            "status": "1"
          },
          {
            "account_id": "4",
            "area_code": "86",
            "change_time": "1748007106",
            "confirm_time": "0",
            "id": "1",
            "mobile": "13800138001",
            "status": "2"
          }
        ],
        "name": {
          "account_id": "4",
          "change_time": "1749898824",
          "id": "3",
          "status": "1",
          "username": "name111"
        },
        "user": {
          "add_time": "1748007106",
          "address_count": "1",
          "change_time": "1748064265",
          "confirm_time": "1748007106",
          "email_count": "1",
          "external_count": "0",
          "id": "4",
          "mobile_count": "2",
          "nickname": "SHAN11",
          "password_id": "7",
          "status": "2",
          "use_name": "1"
        }
      }
    ],
    "next": null
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```


