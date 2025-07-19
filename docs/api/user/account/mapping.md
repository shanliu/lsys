### 获取账户相关状态映射


> 响应参数

| 参数名                      | 类型   | 说明           |
|---------------------------|--------|----------------|
| response.account_status   | array  | 账户状态列表    |
| response.account_status.key | string | 状态键值      |
| response.account_status.val | string | 状态说明      |
| response.email_status     | array  | 邮箱状态列表    |
| response.email_status.key | string | 状态键值       |
| response.email_status.val | string | 状态说明       |
| response.mobile_status    | array  | 手机状态列表    |
| response.mobile_status.key | string | 状态键值      |
| response.mobile_status.val | string | 状态说明      |
| response.session_status   | array  | 会话状态列表    |
| response.session_status.key | string | 状态键值      |
| response.session_status.val | string | 状态说明      |

> 示例

```http
POST /api/user/base/mapping
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{}
```


```json
{
  "response": {
    "account_status": [
      {
        "key": "1",
        "val": "初始"
      },
      {
        "key": "2",
        "val": "启用"
      }
    ],
    "email_status": [
      {
        "key": "1",
        "val": "待验证"
      },
      {
        "key": "2",
        "val": "已验证"
      }
    ],
    "mobile_status": [
      {
        "key": "1",
        "val": "待验证"
      },
      {
        "key": "2",
        "val": "已验证"
      }
    ],
    "session_status": [
      {
        "key": "1",
        "val": "正常"
      },
      {
        "key": "2",
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