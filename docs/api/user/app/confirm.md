### 审核申请请求

> 请求参数

| 参数名         | 类型   | 必填 | 描述           |
|----------------|--------|------|----------------|
| app_req_id     | number | 是   | 申请请求ID     |
| confirm_status | number | 是   | 确认状态       |
| confirm_note   | string | 否   | 确认备注       |


> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
### 审核
POST /api/user/app/confirm
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_req_id": 18,
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

