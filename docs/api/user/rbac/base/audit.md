### RBAC审计日志
> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_ip | string | 否 | 用户IP地址 |
| device_id | string | 否 | 设备ID |
| request_id | string | 否 | 请求ID |
| res_data | string | 否 | 资源数据 |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | string | 否 | 起始位置 |
| limit.limit | int | 否 | 每页数量 |
| limit.forward | boolean | 否 | 是否向前查询 |
| limit.more | boolean | 否 | 是否获取更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| data.audit.add_time | int | 添加时间(秒) |
| data.audit.check_result | string | 检查结果 |
| data.audit.device_id | string | 设备ID |
| data.audit.device_name | string | 设备名称 |
| data.audit.id | string | 审计ID |
| data.audit.request_id | string | 请求ID |
| data.audit.role_key_data | string | 角色键值数据 |
| data.audit.token_data | string | 令牌数据 |
| data.audit.user_app_id | string | 用户应用ID |
| data.audit.user_id | string | 用户ID |
| data.audit.user_ip | string | 用户IP |
| data.detail.add_time | int | 详情添加时间(秒) |
| data.detail.check_result | string | 详情检查结果 |
| data.detail.id | string | 详情ID |
| data.detail.is_role_all | string | 是否所有角色 |
| data.detail.is_role_excluce | string | 是否排除角色 |
| data.detail.is_role_include | string | 是否包含角色 |
| data.detail.is_root | string | 是否根权限 |
| data.detail.is_self | string | 是否自身权限 |

> 示例

```http
POST /api/user/rbac/base/audit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "user_ip": null,
    "device_id":null,
    "request_id":null,
    "res_data":null,
    "count_num":true,
    "limit":{
        "pos":null,
        "limit":10,
        "forward":false,
        "more":true
    }
}
```

```json
{
  "response": {
    "data": [
      {
        "audit": {
          "add_time": "1749867295",
          "check_result": "1",
          "device_id": "",
          "device_name": "httpyac",
          "id": "3412",
          "request_id": "a19mmwq5oxibbkmr",
          "role_key_data": "[{\"key\":\"system-global\",\"user_id\":0},{\"key\":\"system-login\",\"user_id\":0}]",
          "token_data": "dgrsgocyznqdjsvyifcbywrnylxaskvd",
          "user_app_id": "0",
          "user_id": "7",
          "user_ip": "127.0.0.1"
        },
        "detail": [
          {
            "add_time": "1749867295",
            "check_result": "1",
            "id": "3630",
            "is_role_all": "0",
            "is_role_excluce": "0",
            "is_role_include": "0",
            "is_root": "1",
            "is_self": "0",
            "op_id": "0",
            "op_key": "app-sms-config",
            "rbac_audit_id": "3412",
            "res_data": "",
            "res_id": "0",
            "res_type": "global-user",
            "res_user_id": "7",
            "role_data": "[]"
          }
        ]
      }
    ],
    "next": "3402",
    "total": "433"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```