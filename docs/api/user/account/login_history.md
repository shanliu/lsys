# 获取登录历史记录

> 请求参数

| 参数名        | 类型    | 必填 | 说明                     |
|--------------|---------|------|-------------------------|
| login_type   | string  | 否   | 登录类型                 |
| login_account| string  | 否   | 登录账号                 |
| login_ip     | string  | 否   | 登录IP                  |
| is_login     | boolean | 否   | 是否登录成功             |
| count_num    | boolean | 否   | 是否返回总数             |
| limit.pos    | int     | 否   | 起始位置                 |
| limit.limit  | int     | 否   | 每页数量                 |
| limit.forward| boolean | 否   | 是否向前查询             |
| limit.more   | boolean | 否   | 是否查询更多             |

> 响应参数

| 参数名                  | 类型    | 说明                |
|-----------------------|---------|-------------------|
| response.data         | array   | 历史记录数据数组     |
| response.data.account_id | string | 账户ID           |
| response.data.add_time | string | 添加时间(秒)       |
| response.data.id      | string  | 记录ID            |
| response.data.is_login | string | 是否登录成功       |
| response.data.login_account | string | 登录账号     |
| response.data.login_city | string | 登录城市        |
| response.data.login_ip | string | 登录IP           |
| response.data.login_msg | string | 登录消息        |
| response.data.login_type | string | 登录类型       |
| response.next        | string  | 是否有下一页        |
| response.total       | string  | 总记录数           |
| result.code          | string  | 响应代码           |
| result.message       | string  | 响应消息           |
| result.state         | string  | 响应状态           |

> 示例

```http
POST /api/user/base/login_history
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
    "login_type":null,
    "login_account": null,
    "login_ip": null,
    "is_login":null,
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
        "account_id": "1",
        "add_time": "1749479465",
        "id": "32",
        "is_login": "1",
        "login_account": "name",
        "login_city": "",
        "login_ip": "127.0.0.1",
        "login_msg": "",
        "login_type": "name"
      }
    ],
    "next": "1",
    "total": "11"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```