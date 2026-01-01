### 添加阿里云短信模板配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| name | string | 是 | 配置名称 |
| ali_config_id | int | 是 | 阿里云配置ID |
| tpl_key | string | 是 | 模板标识 |
| aliyun_sms_tpl | string | 是 | 阿里云短信模板ID |
| aliyun_sign_name | string | 是 | 阿里云短信签名 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/ali_app_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
   "name": "xxx",
    "ali_config_id":7,
    "tpl_key": "adfad",
    "aliyun_sms_tpl": "adfad",
    "aliyun_sign_name": "adfad"
}
```

```json
{
  "response": {
    "id": "26"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

