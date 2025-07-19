### 添加操作

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| op_key | string | 操作key |
| op_name | string | 操作名称 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 操作ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /rest/rbac/op?method=add
Content-type:application/json

{
  "use_app_user":false,
    "user_param": "account_11",
    "op_key": "xx5",
    "op_name": "xx5"
}
```

```json
{
  "response": {
    "id": "9"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```