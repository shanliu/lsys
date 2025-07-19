### 获取用户地址列表

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.data.address_code | string | 地区编码 |
| response.data.address_detail | string | 详细地址 |
| response.data.address_info | string | 地区信息 |
| response.data.change_time | int | 修改时间 |
| response.data.code_detail.code | string | 地区编码 |
| response.data.code_detail.name | string | 地区名称 |
| response.data.country_code | string | 国家编码 |
| response.data.id | string | 地址ID |
| response.data.mobile | string | 手机号码 |
| response.data.name | string | 收件人姓名 |
| response.total | string | 总数 |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
POST /api/user/profile/address/list_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{}
```

```json
{
  "response": {
    "data": [
      {
        "address_code": "111000",
        "address_detail": "布吉xxx",
        "address_info": "广东深圳龙岗",
        "change_time": "1748179120",
        "code_detail": [
          {
            "code": "11",
            "name": "北京市"
          }
        ],
        "country_code": "CHN",
        "id": "1",
        "mobile": "13500135000",
        "name": "xxx"
      }
    ],
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

