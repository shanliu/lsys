
### 应用申请列表

> 请求参数

| 参数名 | 类型 | 是否必须 | 说明 |
|--------|------|----------|------|
| app_id | string | 否 | 应用ID |
| status | string | 否 | 状态 |
| count_num | boolean | 否 | 是否统计总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.change_data.client_id | string | 客户端ID |
| response.data.change_data.name | string | 应用名称 |
| response.data.confirm_note | string | 确认备注 |
| response.data.confirm_time | int | 确认时间 |
| response.data.confirm_user_id | string | 确认用户ID |
| response.data.feature_data | object | 特性数据 |
| response.data.id | string | 记录ID |
| response.data.oauth_client_data | object | OAuth客户端数据 |
| response.data.request_time | int | 请求时间 |
| response.data.request_type | string | 请求类型 |
| response.data.request_user_id | string | 请求用户ID |
| response.data.status | string | 状态 |
| response.data.user_data.app_id | string | 用户所属应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户数据 |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.total | int | 总记录数 |
| result.code | int | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/app/request_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":null,
   "status":null,
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
        "app_id": "6",
        "change_data": {
          "client_id": "testapp",
          "name": "测试APP"
        },
        "confirm_note": "ssss",
        "confirm_time": "1749032350",
        "confirm_user_id": "1",
        "feature_data": {
          "feature": "feature-mail,feature-sms,feature-barcode"
        },
        "id": "8",
        "oauth_client_data": {
          "scope_data": [
            "mail1"
          ]
        },
        "request_time": "1748930395",
        "request_type": "1",
        "request_user_id": "1",
        "status": "2",
        "user_data": {
          "app_id": "0",
          "id": "1",
          "user_account": "aaaaa",
          "user_data": "1",
          "user_nickname": "root"
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