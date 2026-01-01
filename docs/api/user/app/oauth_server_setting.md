### OAuth服务器设置

> 请求参数

| 参数名 | 类型 | 是否必填 | 说明 |
|--------|------|----------|------|
| app_id | int | 是 | 应用ID |
| scope_data | array | 是 | 权限范围数据列表 |
| scope_data.key | string | 是 | 权限标识 |
| scope_data.name | string | 是 | 权限名称 |
| scope_data.desc | string | 是 | 权限描述 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/oauth_server_setting
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id": 1,
    "scope_data":[{
        "key":"mail",
        "name":"邮箱",
        "desc":"获取用户的邮箱地址"
    },{
        "key":"mail1",
        "name":"邮箱1",
        "desc":"获取用户的邮箱地址1"
    }]
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