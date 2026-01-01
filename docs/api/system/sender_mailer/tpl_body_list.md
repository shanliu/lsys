### 邮件模板内容列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 模板ID |
| tpl_id | string | 否 | 模板标识 |
| count_num | boolean | 否 | 是否统计总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 模板列表 |
| response.data.change_time | int | 修改时间（秒） |
| response.data.change_user_id | string | 修改用户ID |
| response.data.id | string | 模板ID |
| response.data.sender_type | string | 发送器类型 |
| response.data.status | string | 状态 |
| response.data.tpl_data | string | 模板内容 |
| response.data.tpl_id | string | 模板标识 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/tpl_body_list
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
        "change_time": "1747988835",
        "change_user_id": "1",
        "id": "2",
        "sender_type": "2",
        "status": "1",
        "tpl_data": "验证码",
        "tpl_id": "valid_code_title",
        "user_id": "0"
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