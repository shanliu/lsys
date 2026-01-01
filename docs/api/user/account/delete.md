### 删除账户

> 请求参数

| 参数名   | 类型   | 必填 | 说明   |
|----------|--------|------|--------|
| password | string | 是   | 密码   |

> 响应参数

| 参数名         | 类型   | 说明     |
|----------------|--------|----------|
| result.code    | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state   | string | 响应状态 |

> 示例

```http
POST /api/user/base/delete
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
    "password": "login_account111"

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