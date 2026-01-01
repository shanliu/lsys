### 角色列表

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| role_name | string | 角色名称过滤 |
| role_key | string | 角色key过滤 |
| user_data | string | 用户数据过滤 |
| user_count | boolean | 是否返回用户数量 |
| ids | array | ID过滤数组 |
| user_range | int | 用户范围过滤 |
| res_range | int | 资源范围过滤 |
| res_count | int | 是否返回关联资源数量 |
| res_op_count | int | 是否返回关联授权数量 |
| count_num | boolean | 是否返回总数 |
| page.page | int | 页码 |
| page.limit | int | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.count | string | 总数 |
| response.data | array | 数据列表 |
| response.data.change_time | int | 修改时间 |
| response.data.id | string | 角色ID |
| response.data.res_range | string | 资源范围 |
| response.data.role_key | string | 角色key |
| response.data.role_name | string | 角色名称 |
| response.data.user_count | int | 用户数量 |
| response.data.user_range | string | 用户范围 |
| response.data.user_data | object | 用户数据 |
| response.data.user_data.app_id | string | 用户应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户标识 |

> 示例

```http
POST /rest/rbac/role?method=list
Content-type:application/json

{
  "use_app_user":false,
     "user_param": "account_11",
   "role_name":null,
    "role_key":null,
    "user_data":null,
     "res_count":true,
    "res_op_count":true,
    "user_count":true,
    "ids":null,
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
    "count": "1",
    "data": [
      {
        "change_time": "1749736993",
        "id": "13",
        "res_range": "1",
        "role_key": "",
        "role_name": "xxx",
        "user_count": null,
        "user_data": {
          "app_id": "16",
          "id": "86",
          "user_account": "\u0000**\u0000",
          "user_data": "account_11",
          "user_nickname": "a**1"
        },
        "user_id": "86",
        "user_list": null,
        "user_range": "1"
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
