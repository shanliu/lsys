### 查看指定子应用的秘钥信息

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| app_secret | bool | 否 | 是否返回应用密钥 |
| oauth_secret | bool | 否 | 是否返回OAuth密钥 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.app_secret.secret_data | string | 应用密钥数据 |
| response.app_secret.time_out | int | 应用密钥超时时间 |
| response.notify_secret.secret | string | 回调通知密钥 |
| response.notify_secret.timeout | int | 通知密钥超时时间 |
| response.oauth_secret.secret_data | string | OAuth密钥数据 |
| response.oauth_secret.time_out | int | OAuth密钥超时时间 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/sub_app_secret_view
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 5,
   "app_secret": true,
   "oauth_secret": true
}

```

```json
{
  "response": {
    "app_secret": [
      {
        "secret_data": "482a5edc4b943eb9796ed7492d3a1df3",
        "time_out": "0"
      }
    ],
    "notify_secret": {
      "secret": "545e8f1b9ea7009140c62fe687372a00",
      "timeout": "0"
    },
    "oauth_secret": [
      {
        "secret_data": "d8d56504ed0720559d111b537769461b",
        "time_out": "0"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```