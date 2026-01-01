### 获取当前登陆用户的全局动态资源类型


> 示例

```http
POST /api/user/rbac/res/dynamic_res_type
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "res_type":"xxx",
   "count_num":true,
    "page":{
      "page":1,
      "limit":10
   }
}

```