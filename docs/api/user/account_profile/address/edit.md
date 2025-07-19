
### 编辑用户地址

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| address_id | int | 是 | 地址ID |
| code | string | 是 | 地区编码 |
| info | string | 是 | 地区信息 |
| detail | string | 是 | 详细地址 |
| name | string | 是 | 收件人姓名 |
| mobile | string | 是 | 手机号码 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |




```http
POST /api/user/profile/address/edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "address_id":1,
    "code": "111000",
    "info": "广东深圳龙岗",
    "detail": "布吉xxx",
    "name": "xxx",
    "mobile":"13500135000"
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

