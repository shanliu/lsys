### OAuth登录申请scope权限审核


> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_req_id | int | 是 | 申请ID |
| confirm_status | int | 是 | 审核状态 |
| confirm_note | string | 否 | 审核备注 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 返回码 |  
| result.message | string | 返回消息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/system/app/oauth_client_scope_confirm
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_req_id": 33,
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