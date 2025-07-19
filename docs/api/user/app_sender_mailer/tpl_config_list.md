### 获取邮件模板配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID |
| app_id | int | 是 | 应用ID |
| tpl | string | 否 | 模板标识 |
| app_info | boolean | 否 | 是否返回应用信息 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.app_id | string | 应用ID |
| response.data.change_time | string | 修改时间(秒) |
| response.data.change_user_id | string | 修改用户ID |
| response.data.config_data.body_tpl_id | string | 内容模板ID |
| response.data.config_data.from_email | string | 发件人邮箱 |
| response.data.config_data.reply_email | string | 回复邮箱 |
| response.data.config_data.subject_tpl_id | string | 主题模板ID |
| response.data.id | string | 配置ID |
| response.data.name | string | 配置名称 |
| response.data.setting_id | string | SMTP配置ID |
| response.data.setting_key | string | SMTP配置类型 |
| response.data.setting_name | string | SMTP配置名称 |
| response.data.tpl_key | string | 模板标识 |
| response.data.user_id | string | 创建用户ID |
| response.total | string | 总数 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/tpl_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "id": null,
   "app_id":16,
   "tpl":null,
   "app_info":null,
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
        "app_id": "16",
        "change_time": "1749875571",
        "change_user_id": "7",
        "config_data": {
          "body_tpl_id": "test1",
          "from_email": "rustlang@qq.com",
          "reply_email": "rustlang1@qq.com",
          "subject_tpl_id": "111"
        },
        "id": "18",
        "name": "test12",
        "setting_id": "4",
        "setting_key": "smtp-config",
        "setting_name": "111we3",
        "tpl_key": "ddddd",
        "user_id": "7"
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