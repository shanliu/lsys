### 获取可用于申请子应用的的父应用列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| key_word | string | 否 | 搜索关键字 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.change_time | int | 修改时间 |
| response.data.client_id | string | 客户端ID |
| response.data.id | string | 父应用ID |
| response.data.name | string | 父应用名称 |
| response.data.status | string | 状态 |
| response.data.user_data.app_id | string | 用户应用ID |
| response.data.user_data.id | string | 用户ID |
| response.data.user_data.user_account | string | 用户账号 |
| response.data.user_data.user_data | string | 用户数据 |
| response.data.user_data.user_nickname | string | 用户昵称 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 消息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app/parent_app
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "key_word": null,
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
        "change_time": "1748403930",
        "client_id": "dd223323ss",
        "id": "2",
        "name": "ddff23722322",
        "status": "2",
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
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```