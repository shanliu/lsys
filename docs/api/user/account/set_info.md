### 设置账户信息

> 请求参数

| 参数名    | 类型   | 必填 | 说明           |
|-----------|--------|------|----------------|
| nikename  | string | 否   | 昵称           |
| gender    | int    | 否   | 性别           |
| headimg   | string | 否   | 头像URL        |
| birthday  | string | 否   | 生日(YYYY-MM-DD)|

> 响应参数

| 参数名         | 类型   | 说明     |
|----------------|--------|----------|
| result.code    | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state   | string | 响应状态 |

> 示例

```http
POST /api/user/base/set_info
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
  "nikename":"xxx",
    "gender":1,
    "headimg":"xxx",
    "birthday":"2028-11-11"
}
```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```