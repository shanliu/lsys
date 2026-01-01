### 添加资源操作

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| op_key | string | 操作权限键值 |
| op_name | string | 操作权限名称 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 新增操作权限ID |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/op/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "op_key": "xx113",
    "op_name": "xx223"
}

```

```json
{
  "response": {
    "id": "4"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```