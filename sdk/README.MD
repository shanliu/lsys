## 接口文档

> 目前已提供 go 封装,可参考[示例](go/examples/)




## 请求参数及请求签名生成说明(非OAUTH)

> 当 Content-type 为 application/json 时 将尝试把请求内容[如:POST内容]做为JSON字符串并解析为接口参数

> 当 Content-type 非 application/json 时 将尝试把GET参数中的 payload 做为JSON并解析为接口参数



###  公共请求HEADER:

| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |
|---------------|---------|------|------|----------------|------------|
| X-Request-ID	 | String	 | 否		  | 32		 | 请求ID,每次请求不同			 | xxxxx12456 |


### 公共参数:

> 以下参数通过GET形式传递

> 当 Content-type:application/json 时,POST参数为JSON字符串,并作为 `GET payload` 参数代替

| 参数         | 类型      | 是否必填 | 最大长度  | 描述                               | 示例值                                  |
|------------|---------|------|-------|----------------------------------|--------------------------------------|
| app_id     | String	 | 是		  | 32		  | 应用ID                             | test1                                |
| version	   | String	 | 是		  | 3		   | 调用的接口版本                          | 固定为：2.0                              |
| timestamp	 | String	 | 是		  | 19		  | 发送请求的时间,格式"yyyy-MM-dd HH:mm:ss"	 | 2014-07-24 03:07:50                  |
| sign		     | String	 | 是		  | 32		  | 请求参数的签名串	                        | 生成方式参见`签名生成`                         |
| request_ip | String	 | 否		  | 40		  | 客户端IP,存在时加入签名                    | 127.0.0.1                            |
| method		   | String	 | 否		  | 128		 | 接口名称,可以放到URL中,存在时加入签名            | product.detail                       |
| token	     | String	 | 否		  | 无		   | OAUTH登录后获取TOKEN,存在时需加入签名              | MTQtSldES1RIUVVZT0NGUkVNUEdJQlpBTlhM |
| payload	     | String	 | 否		  | 无		   | Content-type 非 application/json 时的作为接口参数,内容为JSON字符串      | {"id":1} |

####  签名生成方法及示例:

##### 签名涉及参数

> 使用 app,version,timestamp,request_ip,method,token 及 请求 [`如POST中JSON`或`GET中payload`] 生成秘钥.
> 其中 request_ip,method,token 及 [`如POST中JSON`或`GET中payload`] 为可选值

##### 签名生成示例

>  假设签名秘钥
```
3f95638a1e07b87df2b64e09c2541dac
```

---

> `假设Content-type 为 application/json的POST请求:`
```
GET:app_id=1212f&version=2.0&timestamp=2023-04-24 15:36:20&method=view&request_ip=fe80::e1bd:c78d:610f:3d03
POST:{"client_id":"1212f"}
```

> 把GET参数排序加payload跟秘钥,生成签名
```
app_id=1212f&method=view&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&timestamp=2023-04-24+15%3A36%3A20&version=2.0{"client_id":"1212f"}3f95638a1e07b87df2b64e09c2541dac
```

> 生成签名(转为小写)
```
d5d21befc41d017064e28a807ecd65b6
```

> 实际请求参数(POST&&Content-type:application/json)
```
GET:app_id=1212f&version=2.0&timestamp=2023-04-24+15%3A36%3A20&method=view&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&sign=d5d21befc41d017064e28a807ecd65b6
POST:{"client_id":"1212f"}
```
---



> `假设Content-type 非 application/json的GET请求(其中GET存在可选参数 request_ip,payload):`
```
GET:app_id=1212f&payload={"client_id":"1212f"}&request_ip=fe80::e1bd:c78d:610f:3d03&timestamp=2023-04-24:15:45:22&version=2.0
```

> 把GET参数排序加payload跟秘钥,生成签名
```
app_id=1212f&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&timestamp=2023-04-24+15%3A45%3A22&version=2.0{"client_id":"1212f"}3f95638a1e07b87df2b64e09c2541dac
```

> 生成签名(转为小写)

```
8fea66dc4b9928fa0664cbe06947e630
```

> 实际请求参数(GET)
```
GET:app_id=1212f&payload=%7B%22client_id%22%3A%221212f%22%7D&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&timestamp=2023-04-24+15%3A45%3A22&version=2.0&sign=8fea66dc4b9928fa0664cbe06947e630
```
---




## OAUTH 请求参数

### 登录地址获取

> 请求路径为: ${OAUTH_HOST}/oauth.html

> 重定向到生成的路径

| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |
|---------------|---------|------|------|----------------|------------|
| client_id  	 | String	 | 是		  | 32		 | 应用ID,跟非OAUTH的参数app_id相同| xxxxx12456 |
| redirect_uri  	 | String	 | 是		  | 32		 | 登录后重定向地址,必须跟后台配置的域名匹配 | https://127.0.0.1:8080 |
| response_type  	 | String	 | 是		  | 4		 | 传入 code| code |
| state  	 | String	 | 否		  | 32		 | 随机值,完成登录原样返回| 23dfa |
| scope  	 | String	 | 是		  | 64		 | 授权功能,多个逗号分割:user_info 用户信息 	user_email 用户邮箱 user_mobile用户手机号  | user_info |


####  登录地址获取示例:

>  生成授权地址
```
http://175.178.90.181/ui/oauth.html?client_id=1212f&redirect_uri=http%3A%2F%2F127.0.0.1%3A8080%2F&response_type=code&scope=user_info&state=aa
```

> 授权完成后返回
```
http://127.0.0.1:8080/?code=27b5591cb788309dfee63da4fc264a10&state=aa
```


### 通过code获取授权token

> 请求路径为: ${APP_HOST}/oauth/token

> 无需签名,请求方式为GET

| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |
|---------------|---------|------|------|----------------|------------|
| client_id  	 | String	 | 是		  | 32		 | 应用ID,跟非OAUTH的参数app_id相同| xxxxx12456 |
| client_secret  	 | String	 | 是		  | 32		 |OAuthSecret,从后台获取,注意:不是AppSecret | 2a97bf1b4f075b0ca7467e7c6b223f89 |
| code  	 | String	 | 是		  | 4		 | 登录后返回的code| 2a97bf1b4f075b0ca7467e7c6b223f89 |

####  获取token示例:

>  请求示例
```
GET:http://175.178.90.181/oauth/token?client_id=1212f&client_secret=2a97bf1b4f075b0ca7467e7c6b223f89&code=27b5591cb788309dfee63da4fc264a10
```

>  返回示例
```
{
	"response": {
		"access_token": "a4985f6747962b0ceb1533a0e28dd1fc",
		"expires_in": "1682929356",
		"openid": "1",
		"refresh_token": null,
		"scope": "user_info"
	},
	"result": {
		"code": "200",
		"message": "ok",
		"state": "ok"
	}
}
```


### 刷新授权token

> 请求路径为: ${APP_HOST}/oauth/refresh_token

> 无需签名,请求方式为GET

| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |
|---------------|---------|------|------|----------------|------------|
| client_id  	 | String	 | 是		  | 32		 | 应用ID,跟非OAUTH的参数app_id相同| xxxxx12456 |
| client_secret  	 | String	 |是		  | 32		 |OAuthSecret,从后台获取,注意:不是AppSecret | 2a97bf1b4f075b0ca7467e7c6b223f89 |
| refresh_token  	 | String	 | 是		  | 4		 | 通过oauth/token获取的TOKEN| 2a97bf1b4f075b0ca7467e7c6b223f89 |

####  获取token示例:

>  请求示例
```
GET:http://175.178.90.181/oauth/refresh_token?client_id=1212f&client_secret=2a97bf1b4f075b0ca7467e7c6b223f89&refresh_token=1cbefd9bf60598a17523042eca74836d
```

>  返回示例
```
{
	"response": {
		"access_token": "1cbefd9bf60598a17523042eca74836d",
		"expires_in": "1682931269",
		"openid": "1",
		"refresh_token": "a4985f6747962b0ceb1533a0e28dd1fc",
		"scope": "user_info"
	},
	"result": {
		"code": "200",
		"message": "ok",
		"state": "ok"
	}
}
```

### 获取登录用户信息

> 请求路径为: ${APP_HOST}/oauth/user

> 需签名,参考`请求参数及加密秘钥生成说明(非OAUTH)`

> 请求方式为POST,设置Content-Type:application/json

#### 接口参数说明

> GET参数 method 为 `info` ,其他参数查看`公共参数`说明

> POST参数[JSON序列化]

| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |
|---------------|---------|------|------|----------------|------------|
| user  	 | bool	 | 否		  | 1		 | 用户iD等基本资料|true |
| name  	 | bool	 | 否		  | 1		 | 用户登录名 需要 scope:user_info 授权|true |
| info  	 | bool	 | 否		  | 1		 |用户资料 需要 scope:user_info 授权 | true |
| address  	 | bool	 | 否		  | 1		 |用户收货地址 需要 scope:user_address 授权 | true |
| email  	 | bool	 | 否		  | 1		 |用户邮箱 需要 scope:user_email 授权 | true |
| mobile  	 | bool	 | 否		  | 1		 |用户手机号 需要 scope:user_mobile 授权   | true |


####  获取登录用户示例:

>  请求示例
```
GET:http://175.178.90.181/oauth/user?app_id=1212f&method=info&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&sign=8cdd52847cf6d5ce808c37cfc3d816c3&timestamp=2023-04-24+16%3A46%3A45&token=a4985f6747962b0ceb1533a0e28dd1fc&version=2.0
POST:{"address":false,"email":false,"info":false,"mobile":false,"name":true,"user":true}
```

>  返回示例
```
{
	"response": {
		"user_data": {
			"address": null,
			"email": null,
			"info": null,
			"mobile": null,
			"name": {
				"change_time": 1682318126,
				"id": 1,
				"user_id": 1,
				"username": "aaaaa"
			},
			"user": {
				"add_time": 1667904484,
				"address_count": 0,
				"change_time": 1682318140,
				"confirm_time": 0,
				"email_count": 15,
				"external_count": 5,
				"id": 1,
				"mobile_count": 4,
				"nickname": "测试用户-已开所有权限",
				"password_id": 40,
				"status": 2,
				"use_name": 1
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




## 返回数据说明

> OAUTH跟非OAUTH的请求格式返回一致

###  公共返回HEADER:

| 参数            | 类型      | 是否必填 | 最大长度 | 描述                      | 示例值        |
|---------------|---------|------|------|-------------------------|------------|
| X-Request-ID	 | String	 | 否		  | 32		 | 如果请求时存在原样返回,否则系统内部会生成一个 | xxxxx12456 |


### 公共返回参数说明:

> 当code为200时系统正常,当state为ok时业务无异常

| 参数              | 类型            | 是否必返回 | 最大长度 | 描述                    |
|-----------------|---------------|-------|------|-----------------------|
| result.code	    | String	       | 是			  | 12		 | 系统状态码,除200外均为异常       |
| result.state	   | String	       | 是			  | 12		 | 业务状态,ok 正常,其他可能参见业务说明 |
| result.message	 | String	       | 是			  | 256  | 相关消息                  |
| response		      | Array,Object	 | 是			  | 无    | 接口数据,可能为:{}或[],参见具体接口说明     |

#### 返回参数示例:

```
{
    "result": {
        "code": "200",
        "state": "ok",
        "message": "add success"
    },
    "response": {
		"product":{
			"name":"Iphone",
			"sku":"111_11"
		}
	}
}
```