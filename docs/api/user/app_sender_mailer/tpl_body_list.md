### 获取邮件模板内容列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 内容ID |
| tpl_id | string | 否 | 模板ID |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.change_time | string | 修改时间(秒) |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 内容ID |
| response.data.sender_type | string | 发送器类型 |
| response.data.status | string | 状态 |
| response.data.tpl_data | string | 模板内容 |
| response.data.tpl_id | string | 模板ID |
| response.data.user_id | string | 创建用户ID |
| response.total | string | 总数 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/tpl_body_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "id":null,
   "tpl_id":null,
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
        "change_time": "1749875598",
        "change_user_id": "7",
        "id": "6",
        "sender_type": "2",
        "status": "1",
        "tpl_data": "bad {{code}} aa is: {{aa}}",
        "tpl_id": "test1",
        "user_id": "7"
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