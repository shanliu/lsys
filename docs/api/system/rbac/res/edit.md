### 编辑资源

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| res_id | int | 是 | 资源ID |
| res_name | string | 是 | 资源名称 |
| res_type | string | 是 | 资源类型 |
| res_data | string | 是 | 资源数据 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 返回信息 |
| result.state | string | 状态信息 |

> 示例

```http
POST /api/system/rbac/res/edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "res_id":3,
   "res_name":"11",
   "res_type":"111122",
   "res_data":"1331"
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

