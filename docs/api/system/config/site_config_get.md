### 获取站点配置信息


> 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| config | object | 配置信息对象 |
| config.dis_old_password | int | 禁用旧密码标志 |
| config.site_tips | string | 站点提示信息 |
| config.timeout | int | 超时时间(秒) |
| result | object | 返回结果对象 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/config/site_config/get
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}
```

```json
{
  "response": {
    "config": {
      "dis_old_password": "1",
      "site_tips": "站点提示",
      "timeout": "111"
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```