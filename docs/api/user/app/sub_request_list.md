### 指定应用的子应用请求列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| status | int | 否 | 状态值 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.change_data | object | 请求更改数据 |
| response.data.change_data.client_id | string | 客户端ID |
| response.data.change_data.name | string | 名称 |
| response.data.confirm_note | string | 确认备注 |
| response.data.confirm_time | string | 确认时间(秒) |
| response.data.confirm_user_id | string | 确认用户ID |
| response.data.feature_data | array | 申请外部功能数据数组 |
| response.data.id | string | 记录ID |
| response.data.oauth_client_data.scope_data | array | 申请OAuth登陆授权范围列表 |
| response.data.request_time | string | 请求时间(秒) |
| response.data.request_type | string | 请求类型 |
| response.data.request_user_id | string | 请求用户ID |
| response.data.status | string | 状态 |
| response.data.user_data.app_id | string | 用户应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户数据 |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/user/app/sub_request_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":1,
   "status":null,
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
        "app_id": "5",
        "change_data": {
          "client_id": "dd9319fss",
          "name": "dd11127"
        },
        "confirm_note": "",
        "confirm_time": "0",
        "confirm_user_id": "0",
        "feature_data": ["sms"],
        "id": "18",
        "oauth_client_data": {
          "scope_data": [
            "mail"
          ]
        },
        "request_time": "1749440574",
        "request_type": "6",
        "request_user_id": "1",
        "status": "1",
        "user_data": {
          "app_id": "0",
          "id": "1",
          "user_account": "a**a",
          "user_data": "1",
          "user_nickname": "r**t"
        }
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