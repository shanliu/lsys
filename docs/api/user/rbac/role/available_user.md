
### 获取可用用户列表
> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_data | string | 否 | 用户数据筛选 |
| count_num | boolean | 否 | 是否返回总数 |
| limit.pos | int | 否 | 起始位置 |
| limit.limit | int | 否 | 每页数量 |
| limit.forward | boolean | 否 | 是否向前查询 |
| limit.more | boolean | 否 | 是否获取更多 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| data.app_id | string | 应用ID |
| data.id | string | 用户ID |
| data.user_account | string | 用户账号 |
| data.user_data | string | 用户数据 |
| data.user_nickname | string | 用户昵称 |
| next | string | 下一页标记 |
| total | string | 总数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/rbac/role/available_user
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "user_data": null,
   "count_num":true,
   "limit":{
        "pos":0,
        "limit":10,
        "forward":true,
       "more":true
    }
}
```

```json
{
  "response": {
    "data": [
      {
        "app_id": "0",
        "id": "8",
        "user_account": "s**m",
        "user_data": "5",
        "user_nickname": "S**N"
      }
    ],
    "next": null,
    "total": "5"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
