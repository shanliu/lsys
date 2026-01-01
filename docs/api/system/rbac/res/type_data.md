### 获取资源类型数据统计

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| res_type | string | 否 | 资源类型 |
| count_num | boolean | 否 | 是否统计数量 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页限制数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.res_total | string | 资源总数 |
| response.data.res_type | string | 资源类型 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总记录数 |
| result.code | string | 状态码 |
| result.message | string | 返回消息 |
| result.state | string | 状态标识 |


> 示例

```http
POST /api/system/rbac/res/type_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "res_type":null,
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
        "res_total": "2",
        "res_type": "111122",
        "user_id": "0"
      },
      {
        "app_id": "0",
        "res_total": "1",
        "res_type": "1111224",
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