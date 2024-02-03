
##### 应用管理模块

> 提供服务内应用信息,OAUTH应用及OAUTH登录等

> 接入文档:[README.MD](../../sdk/README.MD)


> 内部模块
1. 使用 RestQuery 获取接口请求
    访问需要:app_id sign version timestamp 等参数
    由RestQuery完成签名校验
2. 登录使用 OauthAuthQuery 获取完成登录校验[可选]
3. 授权查询: 调用Rbac.check 完成必要权限校验[可选]

> 外部模块校验流程
1. 模块自身client_id 自身KEY 访问者client_id
2. 系统返回访问者 key
3. 检查签名,继续完成操作


> oauth 登录过程
1. [用户授权]请求用户 /oauth/authorize?response_type=code&client_id=app_id&redirect_uri=CALLBACK_URL&scope=read
2. [用户授权]显示登录,完成登录,跳到授权页面 以 scope 作为关系KEY,拉取关系角色可操作权限列表
3. [用户授权]完成授权. 生成code ,存放redis, scope,授权时间 +client_id+授权user_id ,设置5分钟超时,回到用户站点 /callback?code=AUTHORIZATION_CODE
4. [用户授权]请求令牌 /oauth/token?client_id=CLIENT_ID&client_secret=CLIENT_SECRET&grant_type=authorization_code&code=AUTHORIZATION_CODE 读取redis 判断client_id,生成token记录并放入本地缓存,返回token
5. [完成鉴权]通过TOKEN访问 参数:[client_id,code,需访问资源名],检查缓存->检查数据库->有结果->检测授权:
	用户授权参数 检查请求资源[yaf_app_token]是否存在 参数: 用户ID[通过APP表可获取] + 关系KEY[yaf_app_token] +(res[请求参数]+授权用户ID[登录TOKEN])
	通过 继续完成内置功能操作



