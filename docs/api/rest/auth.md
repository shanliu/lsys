# REST 认证接口

## 基础信息

- **接口路径**: `/rest/auth`
- **请求方法**: POST
- **请求格式**: JSON
- **响应格式**: JSON

## 请求格式

```json
{
  "method": "接口方法名",
  "param": {
    "参数字段": "参数值"
  }
}
```

---

## 1. 用户登录

**接口方法**: `do_login`

用户通过token_code进行登录认证

### > 请求参数

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| token_code | string | 是 | 登录令牌代码 |
| user_data | string | 是 | 用户数据 |
| user_nickname | string | 是 | 用户昵称 |
| expire_time | int | 是 | 过期时间，单位秒 |
| device_name | string | 否 | 设备名称 |
| user_account | string | 否 | 用户账号 |
| login_ip | string | 否 | 登录IP地址 |
| device_id | string | 否 | 设备ID |
| session_data | object | 否 | 会话数据键值对 |

### > 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| code | int | 状态码，0表示成功 |
| message | string | 返回消息 |
| data.body.token_data | string | 登录后的token令牌数据 |
| data.body.user_id | int | 用户ID |
| data.body.user_nickname | string | 用户昵称 |

---

## 2. 用户登出

**接口方法**: `do_logout`

用户退出登录，清除会话

### > 请求参数

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| token_data | string | 是 | 当前会话的token令牌数据 |

### > 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| code | int | 状态码，0表示成功 |
| message | string | 返回消息 |

---

## 3. 获取登录信息

**接口方法**: `login_info`

获取当前会话的用户信息和会话数据

### > 请求参数

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| token_data | string | 是 | 当前会话的token令牌数据 |

### > 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| code | int | 状态码，0表示成功 |
| message | string | 返回消息 |
| data.body.session | object | 会话对象 |
| data.body.user | object | 用户对象 |

---

## 4. 检查MFA启用状态

**接口方法**: `mfa_is_enabled`

检查一批账号是否启用了MFA双因素认证

### > 请求参数

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| accounts | array[string] | 是 | 账号列表 |

### > 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| code | int | 状态码，0表示成功 |
| message | string | 返回消息 |
| data.body.accounts | array | 账号及启用状态列表 |
| data.body.accounts.account | string | 账号 |
| data.body.accounts.enabled | boolean | 是否启用MFA (true/false) |

---

## 5. 启用MFA绑定

**接口方法**: `mfa_enable`

为指定账号生成和启用MFA绑定，返回绑定后的记录ID

### > 请求参数

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| user_data | string | 是 | 用户数据 |
| secret | string | 是 | Base32编码的TOTP密钥 |

### > 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| code | int | 状态码，0表示成功 |
| message | string | 返回消息 |
| data.body.record_id | int | MFA绑定记录ID |
| data.body.user_data | string | 用户数据 |

---

## 6. 验证MFA码

**接口方法**: `mfa_verify`

验证指定账号的MFA双因素认证码

### > 请求参数

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| user_data | string | 是 | 用户数据 |
| code | string | 是 | TOTP验证码，6-8位数字 |

### > 响应参数

| 参数 | 类型 | 说明 |
|------|------|------|
| code | int | 状态码，0表示成功 |
| message | string | 返回消息 |
| data.body.user_data | string | 用户数据 |

---

## 权限说明

- 接口 1-3（登录/登出/登录信息）：需要应用启用外部登录功能
- 接口 4-6（MFA相关）：需要RBAC权限检查和应用用户权限验证

## 错误说明

- 权限不足：返回RBAC权限相关错误
- 应用未启用外部登录：返回外部登录功能关闭错误
- MFA验证失败：返回MFA错误信息
- 参数验证失败：返回参数解析错误
