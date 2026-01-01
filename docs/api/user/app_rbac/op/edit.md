### 编辑操作数据

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| op_id | int | 操作ID |
| op_key | string | 操作键名 |
| op_name | string | 操作名称 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |



> 示例

```http
POST /api/user/app_rbac/op/edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "op_id":12,
    "op_key": "xx1",
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