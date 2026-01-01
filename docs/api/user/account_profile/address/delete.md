
### 删除用户地址

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| address_id | int | 是 | 地址ID |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |
> 示例

```http
POST /api/user/profile/address/delete
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "address_id": 4
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
