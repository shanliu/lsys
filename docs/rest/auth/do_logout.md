### 系统应用退出登录接口

> payload参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| token_data | string | 是 | 登录令牌 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |  
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /rest/auth?method=do_logout
Content-type:application/json

{
    "token_data": "3595aff6d32e74bffa93a42785dfef2f"
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