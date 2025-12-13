### 查看应用密钥信息

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| days | int | 是  | 倒数天数 |


> 示例

```http
POST /api/user/app/stat
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 1,
   "days": 1
}
```

```json
{
  "response": {
    "data": {
      "notify_data": {
        "all": [
          {
            "date": "2025-12-08",
            "notify_type": "0",
            "status": "0",
            "total": "0"
          },
          {
            "date": "2025-12-08",
            "notify_type": "0",
            "status": "2",
            "total": "0"
          }
        ],
        "success": [
          {
            "date": "2025-12-08",
            "notify_type": "0",
            "status": "2",
            "total": "0"
          }
        ]
      },
      "oauth_access": [
        {
          "date": "2025-12-08",
          "total": "0"
        }
      ],
      "request": {
        "all": [
          {
            "date": "2025-12-08",
            "status": "2",
            "total": "0"
          },
          {
            "date": "2025-12-08",
            "status": "3",
            "total": "0"
          }
        ],
        "processed": [
          {
            "date": "2025-12-08",
            "status": "2",
            "total": "0"
          },
          {
            "date": "2025-12-08",
            "status": "3",
            "total": "0"
          }
        ]
      },
      "sub_app": {
        "all": [
          {
            "date": "2025-12-08",
            "status": "2",
            "total": "0"
          }
        ],
        "enable": [
          {
            "date": "2025-12-08",
            "status": "2",
            "total": "0"
          }
        ]
      }
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```