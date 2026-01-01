### 设置账户密码

> 请求参数

| 参数名       | 类型   | 必填 | 说明     |
|-------------|--------|------|----------|
| old_password| string | 否   | 旧密码   |
| new_password| string | 是   | 新密码   |

> 响应参数

| 参数名         | 类型   | 说明     |
|----------------|--------|----------|
| result.code    | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state   | string | 响应状态 |

> 示例

```http
POST /api/user/base/set_password
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
    "old_password": null,
    "new_password": "login_account"
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