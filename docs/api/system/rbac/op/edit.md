### 修改RBAC操作

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| op_id | int | 是 | 操作ID |
| op_key | string | 是 | 操作键值 |
| op_name | string | 是 | 操作名称 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/op/edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "op_id":3,
    "op_key": "11111",
    "op_name": "xx"
}

```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```