### 应用列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| user_id | int | 否 | 用户ID |
| app_id | int | 否 | 应用ID |
| app_name | string | 否 | 应用名称 |
| status | int | 否 | 状态 |
| client_id | string | 否 | 应用标识 |
| count_num | boolean | 否 | 是否统计数量 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].change_time | int | 修改时间(秒) |
| response.data[].change_user_id | int | 修改用户ID |
| response.data[].client_id | string | 客户端ID |
| response.data[].exter_feature | array | 外部功能列表 |
| response.data[].exter_login | int | 是否支持外部登录 |
| response.data[].id | int | 应用ID |
| response.data[].name | string | 应用名称 |
| response.data[].oauth_client | int | 是否支持OAuth登录 |
| response.data[].oauth_client_data.callback_domain | string | 回调域名 |
| response.data[].oauth_client_data.scope_data | string | 授权范围 |
| response.data[].oauth_server | int | 是否开通OAuth服务功能 |
| response.data[].oauth_server_scope_data[].scope_key | string | 授权范围键名 |
| response.data[].oauth_server_scope_data[].scope_name | string | 授权范围名称 |
| response.data[].status | int | 状态 |
| response.data[].sub_app_count.disable | int | 禁用子应用数量 |
| response.data[].sub_app_count.enable | int | 启用子应用数量 |
| response.data[].sub_app_count.init | int | 初始化子应用数量 |
| response.data[].sup_app | int | 是否开通子应用功能 |
| response.data[].user_data.app_id | int | 用户所属应用ID |
| response.data[].user_data.id | int | 用户ID |
| response.data[].user_data.user_account | string | 用户账号 |
| response.data[].user_data.user_data | string | 用户数据 |
| response.data[].user_data.user_nickname | string | 用户昵称 |
| response.data[].user_id | int | 所属用户ID |
| response.data[].req_count | int | 请求数量 |
| response.total | int | 总数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/app/list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

   "user_id":null,
   "app_id":null,
   "app_name":null,
   "status":null,
   "client_id":null,
   "count_num":true,
   "page":{
      "page":1,
      "limit":100
   }
}
```

```json
{
  "response": {
    "data": [
      {
        "change_time": "1749032350",
        "change_user_id": "1",
        "client_id": "testapp",
        "exter_feature": [
          "mail",
          "sms",
          "barcode"
        ],
        "req_count": "0",
        "exter_login": "0",
        "id": "6",
        "name": "测试APP",
        "oauth_client": "0",
        "oauth_client_data": {
          "callback_domain": "ddd.com",
          "scope_data": "mail"
        },
        "oauth_server": "0",
        "oauth_server_scope_data": [
           {
            "scope_key": "mail",
            "scope_name": "邮箱"
          }
        ],
        "status": "2",
        "sub_app_count": {
          "disable": "0",
          "enable": "0",
          "init": "0"
        },
        "sup_app": "0",
        "user_data": {
          "app_id": "0",
          "id": "1",
          "user_account": "aaaaa",
          "user_data": "1",
          "user_nickname": "root"
        },
        "user_id": "1"
      }
    ],
    "total": "3"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```