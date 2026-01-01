### 确认手机号

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| mobile_id | int | 是 | 手机号ID |
| code | string | 是 | 验证码 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/user/profile/mobile/confirm
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "mobile_id": 2,
    "code":"669336"
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