### 添加用户地址

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| code | string | 是 | 地区编码 |
| info | string | 是 | 地区信息 |
| detail | string | 是 | 详细地址 |
| name | string | 是 | 收件人姓名 |
| mobile | string | 是 | 手机号码 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.id | string | 地址ID |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/user/profile/address/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "code": "441301",
    "info": "广东深圳龙岗",
    "detail": "布吉xxx",
    "name": "xxx",
    "mobile":"13500135000"
}
```

```json
{
  "response": {
    "id": "5"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```