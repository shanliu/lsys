### 角色列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| role_name | string | 否 | 角色名称 |
| role_key | string | 否 | 角色标识 |
| user_data | string | 否 | 用户数据 |
| user_count | boolean | 否 | 是否返回用户数量 |
| res_count | int | 是否返回关联资源数量 |
| res_op_count | int | 是否返回关联授权数量 |
| ids | array | 否 | 角色ID列表 |
| user_range | int | 否 | 用户范围 |
| res_range | int | 否 | 资源范围 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 否 | 页码 |
| page.limit | int | 否 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| data.change_time | int | 修改时间(秒) |
| data.change_user_id | string | 修改用户ID |
| data.id | string | 角色ID |
| data.res_range | string | 资源范围 |
| data.role_key | string | 角色标识 |
| data.role_name | string | 角色名称 |
| data.status | string | 状态 |
| data.user_count | int | 用户数量 |
| data.user_data | string | 用户数据 |
| data.user_id | string | 创建用户ID |
| data.user_range | string | 用户范围 |
| total | string | 总数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/user/rbac/role/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "role_name":null,
    "role_key":null,
    "user_data":null,
    "user_count":true,
    "ids":null,
    "user_range":null,
    "res_range":null,
     "res_count":true,
    "res_op_count":true,
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
        "change_time": "1749868430",
        "change_user_id": "7",
        "id": "22",
        "res_range": "1",
        "role_key": "",
        "role_name": "xxx",
        "status": "1",
        "user_count": null,
        "user_data": null,
        "user_id": "7",
        "user_range": "1"
      }
    ],
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

