
### 列表条件映射

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.app_status | array | 应用状态 |
| response.app_status.key | string | 状态码 |
| response.app_status.val | string | 状态说明 |
| response.request_status | array | 应用申请状态 |
| response.request_status.key | string | 状态码 |
| response.request_status.val | string | 状态说明 |
| response.request_type | array | 应用申请类型 |
| response.request_type.key | string | 类型码 |
| response.request_type.val | string | 类型说明 |
| response.secret_status | array | 密钥状态 |
| response.secret_status.key | string | 状态码 |
| response.secret_status.val | string | 状态说明 |
| result | object | 返回结果 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
### 列表条件
POST /api/system/app/mapping
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
}

```

```json
{
  "response": {
    "app_status": [
      {
        "key": "2",
        "val": "正常"
      },
      {
        "key": "1",
        "val": "正常"
      },
      {
        "key": "3",
        "val": "被禁用"
      }
    ],
    "request_status": [
      {
        "key": "1",
        "val": "待审"
      },
      {
        "key": "2",
        "val": "批准"
      },
      {
        "key": "3",
        "val": "驳回"
      },
      {
        "key": "4",
        "val": "作废"
      }
    ],
    "request_type": [
      {
        "key": "1",
        "val": "新应用申请"
      },
      {
        "key": "2",
        "val": "应该更改申请"
      },
      {
        "key": "3",
        "val": "子应用可用申请"
      },
      {
        "key": "4",
        "val": "外部账号登录系统申请"
      },
      {
        "key": "5",
        "val": "Oauth服务申请"
      },
      {
        "key": "6",
        "val": "Oauth登录申请"
      },
      {
        "key": "7",
        "val": "OAUTH登录新增权限申请"
      },
      {
        "key": "8",
        "val": "外部功能申请:如发短信邮件等"
      }
    ],
    "secret_status": [
      {
        "key": "1",
        "val": "正常"
      },
      {
        "key": "-1",
        "val": "删除"
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