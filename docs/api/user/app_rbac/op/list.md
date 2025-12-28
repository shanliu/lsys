### 应用操作列表

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| app_id | int | 应用ID |
| use_app_user | boolean | 是否使用应用用户 |
| user_param | string | 用户参数 |
| op_name | string | 操作名称 |
| op_key | string | 操作键值 |
| ids | array | ID列表 |
| count_num | boolean | 是否返回总数 |
| res_type_count | boolean | 已关联资源类型数量 |
| check_role_use | boolean | 是否关联角色 |
| page.page | int | 页码 |
| page.limit | int | 每页限制数量 |



> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.count | int | 总数 |
| response.data.app_id | int | 应用ID |
| response.data.change_time | int | 修改时间(秒) |
| response.data.change_user_id | int | 修改用户ID |
| response.data.id | int | 记录ID |
| response.data.op_key | string | 操作键值 |
| response.data.op_name | string | 操作名称 |
| response.data.status | int | 状态 |
| response.data.user_id | int | 用户ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态

> 示例

```http
POST /api/user/app_rbac/op/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
    "use_app_user":false,
    "user_param":"xx",
    "op_name":"xx",
    "op_key":"xx",
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
    "count": "1",
    "data": [
      {
        "app_id": "16",
        "change_time": "1749867152",
        "change_user_id": "7",
        "id": "12",
        "op_key": "xx",
        "op_name": "xx",
        "status": "1",
        "user_id": "91"
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