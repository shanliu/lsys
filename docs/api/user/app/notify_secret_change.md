### 回调密钥变更

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| secret | string | 是 | 密钥 |
| secret_timeout | int | 是 | 超时时间(秒) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | string | 返回的密钥值 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态标识 |

> 示例

```http
POST /api/user/app/notify_secret_change
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 1,
   "secret": "104a3182c67f381f5b753a11648429a6",
   "secret_timeout": 1111
}
```

```json
{
  "response": {
    "data": "104a3182c67f381f5b753a11648429a6"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```