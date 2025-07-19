
### 获取角色权限数据
> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| role_id | int | 是 | 角色ID |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 否 | 页码 |
| page.limit | int | 否 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| data | array | 权限数据列表 |
| total | string | 总数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/rbac/role/perm_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "role_id": 22,
      "count_num":true,
    "page":{
      "page":1,
      "limit":10
   }
}
```