### 获取全局静态资源数据

> 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| response.tpl_data | array | 模板数据列表 |
| response.tpl_data.res_name | string | 资源名称 |
| response.tpl_data.res_type | string | 资源类型 |
| response.tpl_data.op_data | array | 操作数据列表 |
| response.tpl_data.op_data.key | string | 操作键名 |
| response.tpl_data.op_data.name | string | 操作名称 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/rbac/res/static_res_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}
```

```json
{
  "response": {
    "tpl_data": [
      {
        "op_data": [
          {
            "key": "app-mail-config",
            "name": "邮件应用配置"
          }
        ],
        "res_name": "系统后台权限",
        "res_type": "global-system"
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