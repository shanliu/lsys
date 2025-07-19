
### 添加网易云短信模板配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| name | string | 是 | 配置名称 |
| config_id | int | 是 | 网易云配置ID |
| tpl_key | string | 是 | 模板标识 |
| template_id | string | 是 | 模板ID |
| template_map | string | 是 | 模板参数映射 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/netease_app_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
      "app_id":16,
        "name": "xddxddx",
    "config_id":12,
    "tpl_key": "adfdad",
     "template_id": "adfad",
    "template_map": "adfad"
}

```

```json
{
  "response": {
    "id": "29"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```