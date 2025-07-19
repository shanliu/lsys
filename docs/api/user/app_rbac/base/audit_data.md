### 审计数据查询

### 审计数据查询

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_data | object | 否 | 用户数据 |
| app_id | int | 是 | 应用ID |
| user_ip | string | 否 | 用户IP |
| device_id | string | 否 | 设备ID |
| request_id | string | 否 | 请求ID |
| res_data.res_id | int | 是 | 资源ID |
| count_num | boolean | 否 | 是否统计总数 |
| limit.pos | int | 否 | 起始位置 |
| limit.limit | int | 否 | 限制数量 |
| limit.forward | boolean | 否 | 是否向前查询 |
| limit.more | boolean | 否 | 是否查询更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.audit.add_time | int | 添加时间(秒) |
| response.data.audit.check_result | string | 检查结果 |
| response.data.audit.device_id | string | 设备ID |
| response.data.audit.device_name | string | 设备名称 |
| response.data.audit.id | string | 审计ID |
| response.data.audit.request_id | string | 请求ID |
| response.data.audit.role_key_data | string | 角色键数据 |
| response.data.audit.token_data | string | 令牌数据 |
| response.data.audit.user_app_id | string | 用户应用ID |
| response.data.audit.user_id | string | 用户ID |
| response.data.audit.user_ip | string | 用户IP |
| response.data.detail.add_time | int | 明细添加时间(秒) |
| response.data.detail.check_result | string | 明细检查结果 |
| response.data.detail.id | string | 明细ID |
| response.data.detail.is_role_all | string | 是否所有角色 |
| response.data.detail.is_role_excluce | string | 是否排除角色 |
| response.data.detail.is_role_include | string | 是否包含角色 |
| response.data.detail.is_root | string | 是否根用户 |
| response.data.detail.is_self | string | 是否本人 |
| response.data.detail.op_id | string | 操作ID |
| response.data.detail.op_key | string | 操作键名 |
| response.data.detail.rbac_audit_id | string | RBAC审计ID |
| response.data.detail.res_data | string | 资源数据 |
| response.data.detail.res_id | string | 资源ID |
| response.data.detail.res_type | string | 资源类型 |
| response.data.detail.res_user_id | string | 资源用户ID |
| response.data.detail.role_data | string | 角色数据 |
| response.next | string | 下一页标记 |
| response.total | string | 总数 |
| result.code | string | 结果代码 |
| result.message | string | 结果消息 |
| result.state | string | 结果状态 |

> 示例

```http
POST /api/user/app_rbac/base/audit_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "user_data":null,
    "app_id": 16,
    "user_ip": null,
    "device_id":null,
    "request_id":null,
    "res_data": {
      "res_id":9
    },
    "count_num":true,
    "limit":{
        "pos":0,
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
          "add_time": "1749824057",
          "check_result": "1",
          "device_id": "",
          "device_name": "",
          "id": "3351",
          "request_id": "7mxwvn1vzt4p1ork",
          "role_key_data": "[]",
          "token_data": "",
          "user_app_id": "16",
          "user_id": "89",
          "user_ip": "1.1.1.1"
        },
        "detail": [
          {
            "add_time": "1749824057",
            "check_result": "1",
            "id": "3569",
            "is_role_all": "0",
            "is_role_excluce": "0",
            "is_role_include": "1",
            "is_root": "0",
            "is_self": "0",
            "op_id": "11",
            "op_key": "xx5",
            "rbac_audit_id": "3351",
            "res_data": "",
            "res_id": "9",
            "res_type": "xx1",
            "res_user_id": "86",
            "role_data": "[{\"access_timeout\":0,\"access_user_id\":89,\"perm_id\":8,\"role_id\":19,\"role_key\":\"\",\"role_name\":\"xxxp\"}]"
          }
        ]
      }
    ],
    "next": "3354",
    "total": "13"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

