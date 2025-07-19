### 获取邮件发送状态映射

> 请求参数

无

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.log_status | array | 日志状态列表 |
| response.log_status.key | string | 状态键值 |
| response.log_status.val | string | 状态说明 |
| response.log_type | array | 日志类型列表 |
| response.log_type.key | string | 类型键值 |
| response.log_type.val | string | 类型说明 |

> 示例

```http
POST /api/user/app_sender/mailer/mapping
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{

}
```

```json
{
  "response": {
    "log_status": [
      {
        "key": "2",
        "val": "成功"
      },
      {
        "key": "3",
        "val": "失败"
      },
      {
        "key": "5",
        "val": "取消"
      },
      {
        "key": "6",
        "val": "回调成功"
      },
      {
        "key": "7",
        "val": "回调失败"
      }
    ],
    "log_type": [
      {
        "key": "1",
        "val": "新增完成"
      },
      {
        "key": "2",
        "val": "发送日志"
      },
      {
        "key": "3",
        "val": "取消发送"
      }
    ],
    "mail_branch_status": [
      {
        "key": "1",
        "val": "待发送"
      },
      {
        "key": "2",
        "val": "已发送"
      }
    ],
    "mail_config_type": [
      {
        "key": "1",
        "val": "关闭功能"
      },
      {
        "key": "2",
        "val": "频率限制"
      },
      {
        "key": "3",
        "val": "每次最大发送数量"
      },
      {
        "key": "4",
        "val": "指定模板不检测限制"
      },
      {
        "key": "20",
        "val": "指定邮箱屏蔽"
      },
      {
        "key": "21",
        "val": "指定邮箱屏蔽"
      }
    ],
    "mail_send_status": [
      {
        "key": "1",
        "val": "待发送"
      },
      {
        "key": "2",
        "val": "已发送"
      },
      {
        "key": "5",
        "val": "已接收"
      },
      {
        "key": "3",
        "val": "发送失败"
      },
      {
        "key": "4",
        "val": "已取消"
      }
    ],
    "sms_config_type": [
      {
        "key": "1",
        "val": "关闭功能"
      },
      {
        "key": "2",
        "val": "频率限制"
      },
      {
        "key": "3",
        "val": "每次最大发送数量"
      },
      {
        "key": "4",
        "val": "指定模板不检测限制"
      },
      {
        "key": "10",
        "val": "指定号码屏蔽"
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