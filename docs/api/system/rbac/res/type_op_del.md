### 删除资源类型操作

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| res_type | string | 资源类型 |
| op_ids | int[] | 删除操作ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/system/rbac/res/type_op_del
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "res_type":"111122",
   "op_ids":[1]
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