
### 获取RBAC审计数据

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_id | int | 否 | 用户ID |
| app_id | int | 否 | 应用ID |
| user_ip | string | 否 | 用户IP地址 |
| device_id | string | 否 | 设备ID |
| request_id | string | 否 | 请求ID |
| res_data | string | 否 | 资源数据 |
| count_num | boolean | 否 | 是否统计总数 |
| limit.pos | int | 否 | 分页位置 |
| limit.limit | int | 否 | 每页数量 |
| limit.forward | boolean | 否 | 是否向前查询 |
| limit.more | boolean | 否 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 审计数据列表 |
| response.data.audit.id | string | 审计记录ID |
| response.data.audit.add_time | int | 添加时间(秒) |
| response.data.audit.check_result | string | 检查结果 |
| response.data.audit.device_id | string | 设备ID |
| response.data.audit.device_name | string | 设备名称 |
| response.data.audit.request_id | string | 请求ID |
| response.data.audit.role_key_data | string | 角色密钥数据 |
| response.data.audit.token_data | string | 令牌数据 |
| response.data.audit.user_app_id | string | 用户应用ID |
| response.data.audit.user_id | string | 用户ID |
| response.data.audit.user_ip | string | 用户IP |
| response.data.detail | array | 详细审计数据 |
| response.data.detail.id | string | 详细记录ID |
| response.data.detail.add_time | int | 添加时间(秒) |
| response.data.detail.check_result | string | 检查结果 |
| response.data.detail.is_role_all | string | 是否所有角色 |
| response.data.detail.is_role_excluce | string | 是否排除角色 |
| response.data.detail.is_role_include | string | 是否包含角色 |
| response.data.detail.is_root | string | 是否根用户 |
| response.data.detail.is_self | string | 是否自身 |
| response.data.detail.op_id | string | 操作ID |
| response.data.detail.op_key | string | 操作键值 |
| response.data.detail.rbac_audit_id | string | RBAC审计ID |
| response.data.detail.res_data | string | 资源数据 |
| response.data.detail.res_id | string | 资源ID |
| response.data.detail.res_type | string | 资源类型 |
| response.data.detail.res_user_id | string | 资源用户ID |
| response.data.detail.role_data | string | 角色数据 |
| response.next | string | 下一页位置 |
| response.total | string | 总记录数 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/base/audit_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "user_id": null,
    "app_id": null,
    "user_ip": null,
    "device_id":null,
    "request_id":null,
    "res_data": null,
    "count_num":true,
    "limit":{
        "pos":360,
        "limit":10,
        "forward":true,
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
          "add_time": "1748148668",
          "check_result": "1",
          "device_id": "",
          "device_name": "httpyac",
          "id": "370",
          "request_id": "cr3xuc86lylwr6ex",
          "role_key_data": "[{\"key\":\"system-global\",\"user_id\":0},{\"key\":\"system-login\",\"user_id\":0}]",
          "token_data": "LFXPUSVKAGEBOMZIQWRJNYCHDT",
          "user_app_id": "0",
          "user_id": "1",
          "user_ip": "127.0.0.1"
        },
        "detail": [
          {
            "add_time": "1748148668",
            "check_result": "1",
            "id": "589",
            "is_role_all": "0",
            "is_role_excluce": "0",
            "is_role_include": "0",
            "is_root": "1",
            "is_self": "0",
            "op_id": "0",
            "op_key": "view-rbac",
            "rbac_audit_id": "370",
            "res_data": "",
            "res_id": "0",
            "res_type": "global-system",
            "res_user_id": "0",
            "role_data": "[]"
          }
        ]
      }
    ],
    "next": "371",
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```