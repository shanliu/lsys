### 删除操作

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| op_id | int | 操作ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |


> 示例

```http
POST /rest/rbac/op?method=delete
Content-type:application/json

{
   "op_id":5
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