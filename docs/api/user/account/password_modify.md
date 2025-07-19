
### 获取密码修改信息


> 响应参数

| 参数名                | 类型   | 说明           |
|---------------------|--------|----------------|
| response.last_time  | string | 最后修改时间(秒) |
| response.password_timeout | string | 密码是否过期 1:已过期 0:未过期 |
| result.code         | string | 响应代码        |
| result.message      | string | 响应消息        |
| result.state        | string | 响应状态        |

> 示例

```http
POST /api/user/base/password_modify
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}
```

```json
{
  "response": {
    "last_time": "1747934040",
    "password_timeout": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
