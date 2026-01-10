### 获取用户信息接口

> payload参数

| 参数名      | 类型    | 必填 | 说明           |
| ----------- | ------- | ---- | -------------- |
| auth        | bool    | 是   | 是否需要认证   |
| user        | bool    | 是   | 是否返回用户主信息 |
| name        | bool    | 是   | 是否返回用户名信息 |
| info        | bool    | 是   | 是否返回用户详细信息 |
| address     | bool    | 是   | 是否返回地址信息 |
| email       | bool    | 是   | 是否返回邮箱信息 |
| mobile      | bool    | 是   | 是否返回手机号信息 |

> 响应参数

| 参数名                                      | 类型      | 说明                   |
| ------------------------------------------- | --------- | ---------------------- |
| response.user_data.address                  | array     | 用户地址列表           |
| response.user_data.address[].account_id     | string    | 账户ID                 |
| response.user_data.address[].address_code   | string    | 地址编码               |
| response.user_data.address[].address_detail | string    | 详细地址               |
| response.user_data.address[].address_info   | string    | 地址信息               |
| response.user_data.address[].change_time    | int       | 修改时间(秒)           |
| response.user_data.address[].country_code   | string    | 国家代码               |
| response.user_data.address[].id             | string    | 地址ID                 |
| response.user_data.address[].mobile         | string    | 手机号                 |
| response.user_data.address[].name           | string    | 收件人姓名             |
| response.user_data.address[].status         | string    | 状态                   |
| response.user_data.email                    | array     | 用户邮箱列表           |
| response.user_data.email[].account_id       | string    | 账户ID                 |
| response.user_data.email[].change_time      | int       | 修改时间(秒)           |
| response.user_data.email[].confirm_time     | int       | 邮箱确认时间(秒)       |
| response.user_data.email[].email            | string    | 邮箱地址               |
| response.user_data.email[].id               | string    | 邮箱ID                 |
| response.user_data.email[].status           | string    | 状态                   |
| response.user_data.info                     | object    | 用户详细信息           |
| response.user_data.info.account_id          | string    | 账户ID                 |
| response.user_data.info.birthday            | string    | 生日                   |
| response.user_data.info.change_time         | int       | 修改时间(秒)           |
| response.user_data.info.gender              | string    | 性别                   |
| response.user_data.info.headimg             | string    | 头像                   |
| response.user_data.info.id                  | string    | 信息ID                 |
| response.user_data.info.reg_from            | string    | 注册来源               |
| response.user_data.info.reg_ip              | string    | 注册IP                 |
| response.user_data.mobile                   | array     | 用户手机号列表         |
| response.user_data.mobile[].account_id      | string    | 账户ID                 |
| response.user_data.mobile[].area_code       | string    | 区号                   |
| response.user_data.mobile[].change_time     | int       | 修改时间(秒)           |
| response.user_data.mobile[].confirm_time    | int       | 手机号确认时间(秒)     |
| response.user_data.mobile[].id              | string    | 手机号ID               |
| response.user_data.mobile[].mobile          | string    | 手机号                 |
| response.user_data.mobile[].status          | string    | 状态                   |
| response.user_data.name                     | object    | 用户名信息             |
| response.user_data.name.account_id          | string    | 账户ID                 |
| response.user_data.name.change_time         | int       | 修改时间(秒)           |
| response.user_data.name.id                  | string    | 用户名ID               |
| response.user_data.name.status              | string    | 状态                   |
| response.user_data.name.username            | string    | 用户名                 |
| response.user_data.user                     | object    | 用户主信息             |
| response.user_data.user.add_time            | int       | 添加时间(秒)           |
| response.user_data.user.address_count       | int       | 地址数量               |
| response.user_data.user.change_time         | int       | 修改时间(秒)           |
| response.user_data.user.confirm_time        | int       | 确认时间(秒)           |
| response.user_data.user.email_count         | int       | 邮箱数量               |
| response.user_data.user.external_count      | int       | 外部账号数量           |
| response.user_data.user.id                  | string    | 用户ID                 |
| response.user_data.user.mobile_count        | int       | 手机号数量             |
| response.user_data.user.nickname            | string    | 昵称                   |
| response.user_data.user.password_id         | string    | 密码ID                 |
| response.user_data.user.status              | string    | 状态                   |
| response.user_data.user.use_name            | string    | 使用的用户名ID         |
| result.code                                 | string    | 结果码                 |
| result.message                              | string    | 结果信息               |
| result.state                                | string    | 状态                   |

> 示例

```http
POST /oauth/user?method=info&token=nhclyilhisrjezhsaeaskapywmenrbao
Content-type:application/json

{
    "auth":true,
    "user":true,
    "name":true,
    "info":true,
    "address":true,
    "email":true,
    "mobile":true
}
```

```json
{
  "response": {
    "user_data": {
      "address": [
        {
          "account_id": "1",
          "address_code": "111000",
          "address_detail": "布吉xxx",
          "address_info": "广东深圳龙岗",
          "change_time": "1748179120",
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
          "change_time": "1748178519",
          "confirm_time": "1748178596",
          "id": "2",
          "mobile": "13800138004",
          "status": "2"
        }
      ],
      "name": {
        "account_id": "1",
        "change_time": "1748179000",
        "id": "1",
        "status": "1",
        "username": "name"
      },
      "account": {
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

