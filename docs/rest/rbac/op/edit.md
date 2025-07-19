### 编辑操作

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| op_id | int | 操作ID |
| op_key | string | 操作key |
| op_name | string | 操作名称 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |


> 示例

```http
POST /rest/rbac/op?method=edit
Content-type:application/json

{
   "op_id":9,
    "op_key": "xx4",
    "op_name": "xx4"
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


