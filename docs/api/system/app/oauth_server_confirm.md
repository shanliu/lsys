### OAuth服务端申请审核

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| confirm_status | int | 是 | 审核状态: 2 通过 3驳回 |
| confirm_note | string | 是 | 审核备注 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/app/oauth_server_confirm
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 1,
   "confirm_status": 2,
   "confirm_note": "ssss"
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