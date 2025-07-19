### 资源列表查询

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_id | int | 否 | 用户ID |
| res_type | string | 否 | 资源类型 |
| res_data | string | 否 | 资源数据 |
| res_name | string | 否 | 资源名称 |
| ids | array | 否 | ID列表 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页记录数 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.change_time | int | 修改时间 |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 资源ID |
| response.data.res_data | string | 资源数据 |
| response.data.res_name | string | 资源名称 |
| response.data.res_type | string | 资源类型 |
| response.data.status | string | 状态 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总记录数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/res/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "user_id":null,
    "res_type":null,
    "res_data":null,
    "res_name":null,
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
        "change_time": "1749653790",
        "change_user_id": "7",
        "id": "3",
        "res_data": "1331",
        "res_name": "11",
        "res_type": "111122",
        "status": "1",
        "user_id": "0"
      }
    ],
    "total": "4"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```