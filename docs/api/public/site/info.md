### 获取站点基本信息


> 响应参数
| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.exter_type | array | 外部登录支持类型列表 |
| response.site_tips | string | 站点提示信息 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 | 
| result.state | string | 响应状态 |


> 示例

```http
POST /api/site/info
Content-Type: application/json

{

}
```


```json
{
  "response": {
    "exter_type": [
      "wechat"
    ],
    "site_tips": "站点提示"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```