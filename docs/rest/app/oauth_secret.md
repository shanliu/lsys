### 查询子应用OAuth登录信息及秘钥

> payload参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| client_id | string | 是 | 客户端ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.client_id | string | 应用KEY |
| response.name | string | 应用名称 |
| response.secret.secret_data | string | 秘钥数据 |
| response.secret.time_out | int | 秘钥过期时间 |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /rest/app?method=oauth_secret
Content-type:application/json

{
    "client_id": "dd9319fss"
}
```

```json
{
  "response": {
    "client_id": "dd9319fss",
    "name": "dd11127",
    "secret": [
      {
        "secret_data": "5884fa0c24f11d5bf405edd224e5cb92",
        "time_out": "11749450945"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

