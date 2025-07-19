### 获取资源类型操作数据

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| res_type | string | 资源类型 |
| count_num | boolean | 是否返回总数 |
| page.page | int | 页码 |
| page.limit | int | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.change_time | int | 修改时间 |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | ID |
| response.data.op_id | string | 操作ID |
| response.data.res_type | string | 资源类型 |
| response.data.status | string | 状态 |
| response.data.user_data | object | 用户数据 |
| response.data.user_data.app_id | string | 用户应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户标识 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总数 |

> 示例

```http
POST /rest/rbac/res?method=type_op_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "use_app_user":false,
    "user_param": "account_11",
   "res_type":"xx1",
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
        "app_id": "16",
        "change_time": "1749736908",
        "change_user_id": "7",
        "id": "4",
        "op_id": "7",
        "res_type": "xx1",
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