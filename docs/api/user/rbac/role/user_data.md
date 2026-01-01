### 获取角色用户列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| role_id | int | 是 | 角色ID |
| all | boolean | 否 | 是否获取全部数据 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 否 | 页码 |
| page.limit | int | 否 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.change_time | int | 修改时间 |
| response.data.change_user_id | int | 修改人ID |
| response.data.id | int | 记录ID |
| response.data.role_id | int | 角色ID |
| response.data.status | int | 状态 |
| response.data.timeout | int | 超时时间 |
| response.data.user_data.app_id | int | 应用ID |
| response.data.user_data.id | int | 用户ID |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户数据 |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_id | int | 用户ID |
| response.total | int | 总数量 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/rbac/role/user_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "role_id": 22,
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
    "data": [
      {
        "change_time": "1749868526",
        "change_user_id": "7",
        "id": "15",
        "role_id": "22",
        "status": "1",
        "timeout": "0",
        "user_data": {
          "app_id": "0",
          "id": "8",
          "user_account": "s**m",
          "user_data": "5",
          "user_nickname": "S**N"
        },
        "user_id": "8"
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