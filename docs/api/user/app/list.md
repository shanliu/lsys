### 获取应用列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| parent_app_id | string | 否 | 父应用ID |
| app_id | string | 否 | 应用ID |
| status | string | 否 | 状态 |
| client_id | string | 否 | 客户端ID |
| count_num | boolean | 否 | 是否统计数量 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].change_time | int | 修改时间(秒) |
| response.data[].change_user_id | string | 修改用户ID |
| response.data[].client_id | string | 客户端ID |
| response.data[].exter_feature | array | 外部特性 |
| response.data[].exter_login | string | 外部登录 |
| response.data[].id | string | 应用ID |
| response.data[].name | string | 应用名称 |
| response.data[].oauth_client | string | OAuth客户端 |
| response.data[].oauth_client_data | object | OAuth客户端数据 |
| response.data[].oauth_server | string | OAuth服务端 |
| response.data[].oauth_server_scope_data | array | OAuth服务端作用域数据 |
| response.data[].parent_app.id | string | 父应用ID |
| response.data[].parent_app.name | string | 父应用名称 |
| response.data[].parent_app.user_id | string | 父应用用户ID |
| response.data[].status | string | 状态 |
| response.data[].sub_app_count.disable | string | 禁用子应用数量 |
| response.data[].sub_app_count.enable | string | 启用子应用数量 |
| response.data[].sub_app_count.init | string | 初始化子应用数量 |
| response.data[].sup_app | string | 是否为父应用 |
| response.data[].user_data.app_id | string | 用户应用ID |
| response.data[].user_data.id | string | 用户ID |
| response.data[].user_data.user_account | string | 用户账号 |
| response.data[].user_data.user_data | string | 用户数据 |
| response.data[].user_data.user_nickname | string | 用户昵称 |
| response.data[].user_id | string | 用户ID |
| response.total | string | 总数 |
| result.code | string | 状态码 |
| result.message | string | 消息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "parent_app_id":null,
   "app_id":null,
   "status":null,
   "client_id":null,
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
        "change_time": "1748930395",
        "change_user_id": "1",
        "client_id": "testapp",
        "exter_feature": [],
        "exter_login": "0",
        "id": "6",
        "name": "测试APP",
        "oauth_client": "0",
        "oauth_client_data": null,
        "oauth_server": "0",
        "oauth_server_scope_data": [],
        "parent_app":  {
          "id": "1",
          "name": "ddff237222",
          "user_id": "1"
        },
        "status": "1",
        "sub_app_count": {
          "disable": "0",
          "enable": "0",
          "init": "0"
        },
        "sup_app": "0",
        "user_data": {
          "app_id": "0",
          "id": "1",
          "user_account": "a**a",
          "user_data": "1",
          "user_nickname": "r**t"
        },
        "user_id": "1"
      }
    ],
    "total": "6"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

