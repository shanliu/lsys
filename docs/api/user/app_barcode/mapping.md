### 条码映射状态字典

> 响应参数说明

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.create_status | array | 配置状态列表 |
| response.create_status.key | string | 状态键值 |
| response.create_status.val | string | 状态描述 |
| response.parse_status | array | 解析状态列表 |
| response.parse_status.key | string | 状态键值 |
| response.parse_status.val | string | 状态描述 |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |


> 示例

```http
POST /api/user/app_barcode/mapping
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   
}
```

```json
{
  "response": {
    "create_status": [
      {
        "key": "1",
        "val": "私有"
      },
      {
        "key": "2",
        "val": "公开"
      }
    ],
    "parse_status": [
      {
        "key": "1",
        "val": "成功"
      },
      {
        "key": "2",
        "val": "失败"
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