### 操作列表

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| op_name | string | 操作名称过滤 |
| op_key | string | 操作key过滤 |
| ids | array | ID过滤数组 |
| count_num | boolean | 是否返回总数 |
| res_type_count | boolean | 已关联资源类型数量 |
| check_role_use | boolean | 是否关联角色 |
| page.page | int | 页码 |
| page.limit | int | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.count | string | 总数 |
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.change_time | int | 修改时间 |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 操作ID |
| response.data.op_key | string | 操作key |
| response.data.op_name | string | 操作名称 |
| response.data.status | string | 状态 |
| response.data.user_data | object | 用户数据 |
| response.data.user_data.app_id | string | 用户应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户标识 |
| response.data.user_id | string | 用户ID |

> 示例

```http
POST /rest/rbac/op?method=list
Content-type:application/json

{
  "use_app_user":false,
    "user_param":"account_11",
    "op_name":null,
   "op_key":null,
   "ids":null,
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
    "count": "3",
    "data": [
      {
        "app_id": "16",
        "change_time": "1749735534",
        "change_user_id": "7",
        "id": "7",
        "op_key": "xx1",
        "op_name": "xx1",
        "status": "1",
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
