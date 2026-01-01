### 获取操作列表

> 请求参数 

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| op_name | string | 否 | 操作名称 |
| op_key | string | 否 | 操作键名 |
| ids | array | 否 | ID列表 |
| count_num | boolean | 否 | 是否返回总数 |
| res_type_count | boolean | 已关联资源类型数量 |
| check_role_use | boolean | 是否关联角色 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数 

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.app_id | string | 应用ID |
| response.data.change_time | string | 修改时间 |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 操作ID |
| response.data.op_key | string | 操作键名 |
| response.data.op_name | string | 操作名称 |
| response.data.status | string | 状态 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态

> 示例

```http
POST /api/system/rbac/op/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
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
    "data": [
      {
        "app_id": "0",
        "change_time": "1749653830",
        "change_user_id": "7",
        "id": "3",
        "op_key": "11111",
        "op_name": "xx",
        "status": "1"
      }
    ],
    "total": "2"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```