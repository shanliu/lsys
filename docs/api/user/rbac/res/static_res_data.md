### 获取当前登陆用户的全局静态资源数据

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| tpl_data.op_data.op_id | string | 操作ID |
| tpl_data.op_data.op_key | string | 操作标识 |
| tpl_data.op_data.op_name | string | 操作名称 |
| tpl_data.res_name | string | 资源名称 |
| tpl_data.res_type | string | 资源类型 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/user/rbac/res/static_res_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}

```

```json
{
  "response": {
    "tpl_data": [
      {
        "op_data": [
          {
            "op_id": "14",
            "op_key": "view-data",
            "op_name": "查看某资源"
          }
        ],
        "res_name": "用户权限",
        "res_type": "user"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

