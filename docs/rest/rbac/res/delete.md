### 删除资源

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| res_id | int | 资源ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |


> 示例

```http
POST /rest/rbac/res?method=delete
Content-type:application/json

{
   "res_id":7
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