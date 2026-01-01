### 获取资源类型数据

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| use_app_user | boolean | 是否使用app用户 |
| user_param | string | 用户参数 |
| res_type | string | 资源类型过滤 |
| count_num | boolean | 是否返回总数 |
| page.page | int | 页码 |
| page.limit | int | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.res_total | string | 资源总数 |
| response.data.res_type | string | 资源类型 |
| response.data.user_data | object | 用户数据 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总数 |

> 示例

```http
POST /rest/rbac/res?method=type_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "use_app_user":false,
    "user_param": "account_11",
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
        "app_id": "16",
        "res_total": "2",
        "res_type": "xx1",
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
