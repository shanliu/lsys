### 获取角色用户数据

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| role_id | int | 角色ID |
| all | boolean | 是否返回全部 |
| count_num | boolean | 是否返回总数 |
| page.page | int | 页码 |
| page.limit | int | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.count | string | 总数 |
| response.data | array | 数据列表 |
| response.data.change_time | int | 修改时间(秒) |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | ID |
| response.data.role_id | string | 角色ID |
| response.data.status | string | 状态 |
| response.data.timeout | string | 超时时间(秒) |
| response.data.user_data | object | 用户数据 |
| response.data.user_data.app_id | string | 应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户数据 |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_id | string | 用户ID |

> 示例

```http
POST /rest/rbac/role?method=user_data
Content-type:application/json

{
  "use_app_user":false,
     "user_param": "account_11",
      "role_id": 13,
    "all":true,
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
        "change_time": "1749739644",
        "change_user_id": "7",
        "id": "10",
        "role_id": "13",
        "status": "1",
        "timeout": "0",
        "user_data": {
          "app_id": "0",
          "id": "7",
          "user_account": "1**]",
          "user_data": "4",
          "user_nickname": "S**1"
        },
        "user_id": "7"
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