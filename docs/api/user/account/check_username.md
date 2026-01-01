### 验证用户名是否可用

> 请求参数

| 参数名 | 类型   | 必填 | 说明     |
|--------|--------|------|----------|
| name   | string | 是   | 用户名   |

> 响应参数

| 参数名         | 类型   | 说明               |
|----------------|--------|-------------------|
| response.pass  | string | 是否通过 1:通过 0:不通过 |
| result.code    | string | 响应代码           |
| result.message | string | 响应消息           |
| result.state   | string | 响应状态           |

> 示例

```http
POST /api/user/base/check_username
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
    "name": "name1"
}
```


```json
{
  "response": {
    "pass": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```