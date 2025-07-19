# 获取登录信息数据

> 请求参数

| 参数名          | 类型     | 必填 | 说明                    |
|----------------|----------|------|------------------------|
| reload_auth    | boolean  | 否   | 是否重新加载认证信息    |
| auth           | boolean  | 否   | 是否返回认证信息        |
| user           | boolean  | 否   | 是否返回用户信息        |
| name           | boolean  | 否   | 是否返回名称信息        |
| info           | boolean  | 否   | 是否返回详细信息        |
| external       | array    | 否   | 外部登录信息类型数组    |
| email          | array    | 否   | 邮箱信息类型数组        |
| mobile         | array    | 否   | 手机信息类型数组        |
| address        | boolean  | 否   | 是否返回地址信息        |
| password_timeout| boolean | 否   | 是否返回密码超时信息    |

> 响应参数

| 参数名                               | 类型    | 说明                    |
|------------------------------------|---------|------------------------|
| response.auth_data.account_id      | string  | 账户ID                 |
| response.auth_data.empty_password  | string  | 是否为空密码           |
| response.auth_data.login_data.account_id | string | 登录账户ID       |
| response.auth_data.login_data.change_time | string | 更改时间(秒)    |
| response.auth_data.login_data.id   | string  | 登录数据ID             |
| response.auth_data.login_data.status | string | 登录状态              |
| response.auth_data.login_data.username | string | 用户名              |
| response.auth_data.login_time      | string  | 登录时间(秒)           |
| response.auth_data.login_type      | string  | 登录类型               |
| response.auth_data.time_out        | string  | 超时时间(秒)           |
| response.auth_data.user_id         | string  | 用户ID                 |
| response.auth_data.user_nickname   | string  | 用户昵称               |
| response.jwt                       | string  | JWT令牌                |
| response.user_data.address         | array   | 地址信息数组           |

> 示例

```http
POST /api/auth/login_data
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "reload_auth":true,
    "auth":true,
    "user":true,
    "name":true,
    "info":true,
     "external":["wechat"],
    "email":[1,2],
    "mobile":[1,2],
    "address":true,
    "password_timeout":true
}
```

```json
{
  "response": {
    "auth_data": {
      "account_id": "1",
      "empty_password": "0",
      "login_data": {
        "account_id": "1",
        "change_time": "1748179000",
        "id": "1",
        "status": "1",
        "username": "name"
      },
      "login_time": "252002",
      "login_type": "name",
      "time_out": "1749821505",
      "user_id": "1",
      "user_nickname": "xxx"
    },
    "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1Ni",
    "user_data": {
      "address": [
        {
          "account_id": "1",
          "address_code": "111000",
          "address_detail": "布吉xxx",
          "address_info": "广东深圳龙岗",
          "change_time": "1749569169",
          "country_code": "CHN",
          "id": "1",
          "mobile": "13500135000",
          "name": "xxx",
          "status": "1"
        }
      ],
      "email": [
        {
          "account_id": "1",
          "change_time": "1748178278",
          "confirm_time": "1748178492",
          "email": "ssss11121@qq.com",
          "id": "2",
          "status": "2"
        }
      ],
      "external": [],
      "info": {
        "account_id": "1",
        "birthday": "2028-11-11",
        "change_time": "1748179060",
        "gender": "1",
        "headimg": "xxx",
        "id": "4",
        "reg_from": "",
        "reg_ip": ""
      },
      "mobile": [
        {
          "account_id": "1",
          "area_code": "86",
          "change_time": "1748178635",
          "confirm_time": "0",
          "id": "3",
          "mobile": "13800138005",
          "status": "1"
        }
      ],
      "name": {
        "account_id": "1",
        "change_time": "1748179000",
        "id": "1",
        "status": "1",
        "username": "name"
      },
      "passwrod_timeut": "1",
      "user": {
        "add_time": "1747934040",
        "address_count": "3",
        "change_time": "1748179060",
        "confirm_time": "1747934040",
        "email_count": "1",
        "external_count": "0",
        "id": "1",
        "mobile_count": "2",
        "nickname": "xxx",
        "password_id": "1",
        "status": "2",
        "use_name": "1"
      }
    }
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```