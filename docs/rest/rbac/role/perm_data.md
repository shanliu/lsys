### 获取角色权限数据

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| role_id | int | 角色ID |
| count_num | boolean | 是否返回总数 |
| page.page | int | 页码 |
| page.limit | int | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.count | string | 总数 |
| response.data | array | 数据列表 |
| response.data.change_time | int | 修改时间 |
| response.data.change_user_id | string | 修改用户ID |
| response.data.op_id | string | 操作ID |
| response.data.op_key | string | 操作key |
| response.data.op_name | string | 操作名称 |
| response.data.op_status | string | 操作状态 |
| response.data.res_data | string | 资源数据 |
| response.data.res_id | string | 资源ID |
| response.data.res_name | string | 资源名称 |
| response.data.res_status | string | 资源状态 |
| response.data.res_type | string | 资源类型 |
| response.data.user_data | object | 用户数据 |
| response.data.user_data.app_id | string | 用户应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户标识 |
| response.data.user_id | string | 用户ID |

> 示例

```http
POST /rest/rbac/role?method=perm_data
Content-type:application/json

{
  "use_app_user":false,
  "user_param": "account_11",
  "role_id": 19,
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
        "change_time": "1749738356",
        "change_user_id": "16",
        "op_id": "9",
        "op_key": "xx4",
        "op_name": "xx4",
        "op_status": "1",
        "res_data": "xxx3",
        "res_id": "6",
        "res_name": "",
        "res_status": "1",
        "res_type": "xx1",
        "user_data": {
          "app_id": "16",
          "id": "86",
          "user_account": "\u0000**\u0000",
          "user_data": "account_11",
          "user_nickname": "a**1"
        },
        "user_id": "86"
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