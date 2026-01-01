### 查看应用密钥信息

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| app_secret | bool | 否 | 是否获取应用密钥 |
| oauth_secret | bool | 否 | 是否获取OAuth密钥 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.app_secret[].secret_data | string | 应用密钥数据 |
| response.app_secret[].time_out | int | 密钥超时时间 |
| response.notify_secret.secret | string | 应用回调密钥 |
| response.notify_secret.timeout | int | 应用回调密钥超时时间,超时重新生成 |
| response.oauth_secret[].secret_data | string | OAuth密钥数据 |
| response.oauth_secret[].time_out | int | OAuth密钥超时时间 |
| result.code | string | 返回码 |
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |


> 示例

```http
POST /api/user/app/secret_view
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 16,
   "app_secret": true,
   "oauth_secret": true
}
```

```json
{
  "response": {
    "app_secret": [
      {
        "secret_data": "9b2b039e7ced6b78f1f26315b0a24587",
        "time_out": "0"
      }
    ],
    "notify_secret": {
      "secret": "63ae34ca2d01e96c38fd38f14cdb0275",
      "timeout": "0"
    },
    "oauth_secret": [
      {
        "secret_data": "5e0b3f0b8ae636cb7e7c75a43b8952ef",
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