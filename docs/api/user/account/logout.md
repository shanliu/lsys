### 退出登录


> 响应参数

| 参数名         | 类型   | 说明     |
|----------------|--------|----------|
| result.code    | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state   | string | 响应状态 |

> 示例

```http
POST /api/auth/logout
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

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