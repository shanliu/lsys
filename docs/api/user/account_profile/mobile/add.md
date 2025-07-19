### 添加手机号

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| area_code | string | 是 | 区号 |
| mobile | string | 是 | 手机号 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.id | string | 手机号ID |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/user/profile/mobile/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "area_code":"86",
    "mobile": "13800138005"
}
```

```json
{
  "response": {
    "id": "3"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```