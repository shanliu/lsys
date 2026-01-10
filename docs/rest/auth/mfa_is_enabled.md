### 检查MFA启用状态

检查一批账号是否启用了MFA双因素认证

> payload参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| accounts | array[string] | 是 | 账号列表 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.accounts | array | 账号及启用状态列表 |
| response.accounts.account | string | 账号 |
| response.accounts.enabled | boolean | 是否启用MFA (true/false) |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /rest/auth?method=mfa_is_enabled
Content-type:application/json

{
   "accounts": ["user1@example.com", "user2@example.com"]
}
```

```json
{
  "response": {
      "accounts": [
        {
          "account": "user1@example.com",
          "enabled": "1"
        },
        {
          "account": "user2@example.com",
          "enabled": "1"
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
