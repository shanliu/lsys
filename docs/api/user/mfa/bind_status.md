### 获取MFA绑定状态

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.enabled | boolean | 是否已启用MFA（true已启用，false未启用） |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/mfa/bind_status
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}

```

```json
{
  "response": {
    "enabled": true
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
