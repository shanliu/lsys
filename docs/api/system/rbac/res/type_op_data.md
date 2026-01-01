
### 获取资源类型操作数据列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| res_type | string | 是 | 资源类型 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量限制 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.app_id | string | 应用ID |
| response.data.change_time | string | 修改时间 |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 记录ID |
| response.data.op_id | string | 操作ID |
| response.data.res_type | string | 资源类型 |
| response.data.status | string | 状态 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总记录数 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/system/rbac/res/type_op_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "res_type":"global-user",
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
        "change_time": "1749653643",
        "change_user_id": "7",
        "id": "1",
        "op_id": "1",
        "res_type": "111122",
        "status": "1",
        "user_id": "0"
      },
      {
        "app_id": "0",
        "change_time": "1749653643",
        "change_user_id": "7",
        "id": "3",
        "op_id": "3",
        "res_type": "111122",
        "status": "1",
        "user_id": "0"
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