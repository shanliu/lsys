### 邮件模板配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID |
| tpl | string | 否 | 模板标识 |
| app_info | boolean | 否 | 是否返回应用信息 |
| count_num | boolean | 否 | 是否统计总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 配置列表 |
| response.data.app_client_id | string | 应用客户端ID |
| response.data.app_id | string | 应用ID |
| response.data.app_name | string | 应用名称 |
| response.data.change_time | int | 修改时间（秒） |
| response.data.change_user_id | string | 修改用户ID |
| response.data.config_data.body_tpl_id | string | 正文模板ID |
| response.data.config_data.from_email | string | 发件人邮箱 |
| response.data.config_data.reply_email | string | 回复邮箱 |
| response.data.config_data.subject_tpl_id | string | 主题模板ID |
| response.data.id | string | 配置ID |
| response.data.name | string | 配置名称 |
| response.data.setting_id | string | 设置ID |
| response.data.setting_key | string | 设置键值 |
| response.data.setting_name | string | 设置名称 |
| response.data.tpl_key | string | 模板键值 |
| response.data.user_id | string | 用户ID |
| response.total | string | 总数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/tpl_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "id": null,
   "tpl": null,
   "app_info":true,
   "count_num":true,
   "page":{
      "page":1,
      "limit":10
   }
}
```

```json
{
  "response": {
    "data": [
      {
        "app_client_id": "",
        "app_id": "0",
        "app_name": "",
        "change_time": "1747995433",
        "change_user_id": "1",
        "config_data": {
          "body_tpl_id": "valid_code_body",
          "from_email": "rustlang@qq.com",
          "reply_email": "rustlang@qq.com",
          "subject_tpl_id": "valid_code_title"
        },
        "id": "7",
        "name": "邮箱登录",
        "setting_id": "4",
        "setting_key": "smtp-config",
        "setting_name": "111we3",
        "tpl_key": "valid_code_login_email",
        "user_id": "0"
      }
    ],
    "total": "4"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```