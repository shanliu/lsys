### 获取邮箱列表

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| status | int | 否 | 状态过滤 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.data.account_id | string | 账号ID |
| response.data.change_time | int | 修改时间 |
| response.data.confirm_time | int | 确认时间 |
| response.data.email | string | 邮箱地址 |
| response.data.id | string | 邮箱ID |
| response.data.status | string | 邮箱状态 |
| response.total | string | 总数 |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |


> 示例

```http
### list mail
POST /api/user/profile/email/list_data
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
        "change_time": "1748178278",
        "confirm_time": "1748178492",
        "email": "ssss11121@qq.com",
        "id": "2",
        "status": "2"
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