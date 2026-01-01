
### 校验权限接口

此接口用于校验传入的 name 是否被授权。


> 请求参数


| 参数名      | 类型     | 必填 | 说明                         |
| --------- | -------- | ---- | ---------------------------- |
| check_data| array    | 是   | 需校验的权限项数组，每项如下： |
| └─ name   | string   | 是   | 权限名称                      |
| └─ data   | object   | 否   | 额外数据（可选）              |


> 响应参数

| 参数名                 | 类型     | 说明                       |
| ---------------------- | -------- | -------------------------- |
| response.record        | array    | 权限校验结果数组           |
| response.record.name   | string   | 权限名称                   |
| response.record.status | string   | 校验状态，1为有权限，0为无权限 |
| response.record.msg    | string   | 校验信息或错误描述          |
| result.code           | string   | 响应码                      |
| result.message        | string   | 响应信息                    |
| result.state          | string   | 状态                        |


> 请求示例


```http
POST /api/auth/perm/check
Content-Type: application/json
Authorization: Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "check_data": [
    {
      "name": "admin-sms-config",
      "data": null
    },
    {
      "name": "admin-mail-config",
      "data": null
    }
  ]
}
```



> 响应示例

```json
{
  "response": {
    "record": [
      {
        "msg": null,
        "name": "admin-sms-config",
        "status": "1"
      },
      {
        "msg": null,
        "name": "admin-mail-config",
        "status": "1"
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

> 说明

- status = "1" 表示有权限，"0" 表示无权限。
- msg 字段为校验失败时的错误信息，成功时为 null。


