### 获取微信OAuth配置


> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.config.app_id | string | 微信应用ID |
| response.config.app_secret | string | 微信应用密钥 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/config/oauth_config/wechat/get
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}
```

```json
{
  "response": {
    "config": {
      "app_id": "xxx",
      "app_secret": "111"
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```