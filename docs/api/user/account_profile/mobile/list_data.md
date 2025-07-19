### 获取手机号列表

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| status | int | 否 | 状态过滤 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.data.account_id | string | 账号ID |
| response.data.area_code | string | 区号 |
| response.data.change_time | int | 修改时间 |
| response.data.confirm_time | int | 确认时间 |
| response.data.id | string | 手机号ID |
| response.data.mobile | string | 手机号 |
| response.data.status | string | 状态 |
| response.total | string | 总数 |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
### mobile list_data
POST /api/user/profile/mobile/list_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "status": null
}
```

```json
{
  "response": {
    "data": [
      {
        "account_id": "1",
        "area_code": "86",
        "change_time": "1748178635",
        "confirm_time": "0",
        "id": "3",
        "mobile": "13800138005",
        "status": "1"
      }
    ],
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```