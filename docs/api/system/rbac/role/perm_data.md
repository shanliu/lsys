### 角色权限数据列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| role_id | int | 是 | 角色ID |
| count_num | boolean | 否 | 是否统计总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页限制数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.change_time | int | 修改时间 |
| response.data.change_user_id | int | 修改用户ID |
| response.data.op_id | int | 操作ID |
| response.data.op_key | string | 操作键值 |
| response.data.op_name | string | 操作名称 |
| response.data.op_status | int | 操作状态 |
| response.data.res_data | string | 资源数据 |
| response.data.res_id | int | 资源ID |
| response.data.res_name | string | 资源名称 |
| response.data.res_status | int | 资源状态 |
| response.data.res_type | string | 资源类型 |
| response.data.user_id | int | 用户ID |
| response.total | int | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/role/perm_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 11,
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
        "change_time": "1749656289",
        "change_user_id": "7",
        "op_id": "3",
        "op_key": "11111",
        "op_name": "xx",
        "op_status": "1",
        "res_data": "1331",
        "res_id": "3",
        "res_name": "",
        "res_status": "1",
        "res_type": "111122",
        "user_id": "0"
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