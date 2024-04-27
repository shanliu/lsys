## 短信接口文档

> 接入方法及公共参数,可参考[接入文档](rest.md)

> [go实现代码示例](https://github.com/shanliu/lsys/blob/main/sdk/go/examples/basic/sms_test.go)

### 发送短信

> payload参数说明:

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|--------|
| mobile     | []String  | 是       | 接收手机号|
| tpl     | String  | 是       | 模板,在后台创建|
| data     | map[String]String |是 | 内容JSON数据     |
| send_time     | String  | 否      | 发送时间 |
| max_try     | int  | 否     | 失败重试次数|


```http

# @name send_sms
POST /rest/sms?method=send
Content-type:application/json

{
    "mobile":["13800138000"],
    "tpl":"dddd",
    "data":{"code":"11","aa":"111"},
    "send_time":"2024-12-11 10:00:00",
    "max_try":1
}
```

### 取消发送短信


> payload参数说明:

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|--------|
| snid_data     | String  | 是       | 消息ID,发送接口返回|


```http
# @ref send_sms
POST /rest/sms?method=cancel
Content-type:application/json

{
    "snid_data": ["{{send_sms.response.detail[0].snid}}"]
}
```
