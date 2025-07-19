### 获取短信服务状态码映射


> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.log_status[].key | string | 日志状态码 |
| response.log_status[].val | string | 日志状态说明 |
| response.log_type[].key | string | 日志类型码 |
| response.log_type[].val | string | 日志类型说明 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/user/app_sender/smser/mapping
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
    "sms_branch_status": [
      {
        "key": "1",
        "val": "待发送"
      },
      {
        "key": "2",
        "val": "已发送"
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
    ],
    "sms_send_status": [
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
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```