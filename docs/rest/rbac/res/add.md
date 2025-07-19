### 添加资源

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| res_type | string | 资源类型 |
| res_name | string | 资源名称 |
| res_data | string | 资源数据 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 资源ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /rest/rbac/res?method=add
Content-type:application/json

{
  "use_app_user":false,
    "user_param": "account_11",
    "res_type": "xx1",
    "res_name": "xx3",
    "res_data":""
}
```

```json
{
  "response": {
    "id": "6"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

