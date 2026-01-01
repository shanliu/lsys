### 角色列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| role_name | string | 否 | 角色名称 |
| role_key | string | 否 | 角色标识 |
| user_data | string | 否 | 用户数据 |
| user_count | boolean | 否 | 是否统计用户数量 |
| ids | array | 否 | ID列表 |
| user_range | string | 否 | 用户范围 |
| res_range | string | 否 | 资源范围 |
| count_num | boolean | 否 | 是否统计总数 |
| res_count | int | 是否返回关联资源数量 |
| res_op_count | int | 是否返回关联授权数量 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.change_time | int | 修改时间(秒) |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 角色ID |
| response.data.res_range | string | 资源范围 |
| response.data.role_key | string | 角色标识 |
| response.data.role_name | string | 角色名称 |
| response.data.status | string | 状态 |
| response.data.user_count | int | 用户数量 |
| response.data.user_data | string | 用户数据 |
| response.data.user_id | string | 用户ID |
| response.data.user_range | string | 用户范围 |
| response.total | string | 总数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/role/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "role_name":null,
    "role_key":null,
    "user_data":null,
    "user_count":true,
    "ids":null,
    "res_count":true,
    "res_op_count":true,
    "user_range": null,
    "res_range": null,
     "count_num":true,
    "page":{
      "page":1,
      "limit":10
   }
}
```

```json
{
  "response": {
    "data": [
      {
        "change_time": "1749660573",
        "change_user_id": "7",
        "id": "3",
        "res_range": "1",
        "role_key": "",
        "role_name": "xxx13",
        "status": "1",
        "user_count": "4",
        "user_data": null,
        "user_range": "1"
      }
    ],
    "total": "5"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```