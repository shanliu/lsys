### 添加资源

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| res_name | string | 资源名称 |
| res_type | string | 资源类型 |
| res_data | string | 资源数据 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 资源ID |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状|


> 示例

```http
POST /api/system/rbac/res/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "res_name":"133111234",
   "res_type":"11112243",
   "res_data":"2"
}
```

```json
{
  "response": {
    "id": "5"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
