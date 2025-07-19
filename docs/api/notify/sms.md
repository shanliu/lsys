
### 各平台短信发送完回调路径

> ali短信回调

```http
POST /notify/sms/1/111
Content-Type: application/json

[
    {
        "phone_number" : "1381111****",
        "send_time" : "2017-01-01 00:00:00",
        "report_time" : "2017-01-01 00:00:00",
        "success" : true,
        "err_code" : "DELIVERED",
        "err_msg" : "用户接收成功",
        "sms_size" : "1",
        "biz_id" : "12345",
        "out_id" : "67890"
    }
]
```


> ali短信回调

```http
POST /notify/sms/1/111
Content-Type: application/json

[
    {
        "phone_number" : "1381111****",
        "send_time" : "2017-01-01 00:00:00",
        "report_time" : "2017-01-01 00:00:00",
        "success" : true,
        "err_code" : "DELIVERED",
        "err_msg" : "用户接收成功",
        "sms_size" : "1",
        "biz_id" : "12345",
        "out_id" : "67890"
    }
]
```

> CloOpen短信回调

```http
POST /notify/sms/1/111

{
    "Request": {
    "action": "SMSArrived",
    "smsType": "1",
    "apiVersion": "2013-12-26",
    "content": "4121908f3d1b4edb9210f0eb4742f62c",
    "fromNum": "13912345678",
    "dateSent": "20130923010000",
    "deliverCode": "DELIVRD",
    "recvTime": "20130923010010",
    "status": "0",
    "reqId": "123",
    "smsCount": "2",
    "spCode": "10690876"
    }
}
```


> huawei云短信回调

```http
POST /notify/sms/1/111

status=DELIVRD&smsMsgId=xxxxxxxxxxx&updateTime=2018-04-13T06:31:46Z
```

> NetEase云短信回调

```http
POST /notify/sms/1/111
MD5:2d35ef62d088aa6a176ab5e92e30a967
CurTime:2017-06-02 14:40:45
CheckSum:8aee9fa350c6dc7081129794882b4d16bf103000

{"eventType": "11", "objects": [ {  "mobile": "12345678945",  "sendid": "1490",  "result": "DELIVRD",  "sendTime": "2017-06-02 14:40:45",  "reportTime": "2017-06-06 10:40:30",  "spliced": "1","templateId":1234 }, {  "mobile": "12345678945",  "sendid": "1491",  "result": "DELIVRD",  "sendTime": "2017-06-02 14:41:00",  "reportTime": "2017-06-02 10:41:20",  "spliced": "2" ,"templateId":1234} ]}
```

> Tennect云短信回调

```http
POST /notify/sms/1/111

[
    {
        "user_receive_time": "2015-10-17 08:03:04",
        "nationcode": "86",
        "mobile": "13xxxxxxxxx",
        "report_status": "SUCCESS",
        "errmsg": "DELIVRD",
        "description": "用户短信送达成功",
        "sid": "xxxxxxx"
    }
]
```