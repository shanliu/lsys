
##### 应用管理模块

> 提供服务内应用信息,OAUTH应用及OAUTH登录等

> 接入文档:[README.MD](../../sdk/README.MD)


> 内部模块[RUST模块]
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
4. [用户授权]请求令牌 /oauth/token?client_id=CLIENT_ID&client_secret=CLIENT_SECRET&grant_type=authorization_code&code=AUTHORIZATION_CODE 读取redis 判断client_id,生成token记录并放入数据库跟本地缓存,返回token
5. [完成鉴权]通过TOKEN访问 参数:[client_id,code,需访问资源名],检查缓存->检查数据库->有结果->检测授权:
	用户授权参数 检查请求资源[app_token]是否存在 参数: 用户ID[通过APP表可获取] + 关系KEY[app_token] +(res[请求参数]+授权用户ID[登录TOKEN])
	通过 继续完成内置功能操作

id    app_id
1      0
2      1
3      0


id     parent_app_id   user_id        user_app_id
1          0            1[内置账号]    0
2          0            2[内置账号]    0
3          1            3[外部账号]    1
4          1            2[内置账号]    0




(APP=0)平台
建APP:(PAPP=0)
建APP:(PAPP>0)

外部(APP>0):平台
建APP:(PAPP=外部APP)

3. 添加应用【可选上级应用】

ExterLogin + SubApp

a. 内置账号 进入系统,添加app存在以下类型:
    1. 直接访问系统[0:status]
    2. 内部账号oauth登陆[pappid=0,存在AppRequestType::OAuthClient]
    3. 外部账号登陆登陆[pappid=0,存在AppRequestType::ExterLogin]
    4. 内部账号授权  [pappid=0,存在AppRequestType::Featuer+KEY]
    5. 子应用信息查看 [pappid=0,存在AppRequestType::SubApp]
    6. 子应用访问系统【2级-1】 [pappid>0,存在pappid:AppRequestType::Featuer+KEY &&存在AppRequestType::Featuer+KEY]
    7. 子应用oauth登陆[pappid>0,存在papp:AppRequestType::OAuthServer && AppRequestType::OAuthClient]
        1. oauth托管登陆
        2. 自定义oauth登陆
b. 外部账号 通过“子应用登陆”,添加app存在以下类型[父app不为0]:
    1. 子应用访问系统【2级-2】[pappid>0,存在pappid:AppRequestType::Featuer+KEY &&存在AppRequestType::Featuer+KEY]
    2. oauth托管登陆 [pappid>0,存在papp:AppRequestType::OAuthServer && AppRequestType::OAuthClient]
    3. 自定义oauth登陆




父应用[1]
直接访问系统
   可以设置外部登陆登陆
直接oauth登陆系统
直接OAUTH访问授权及数据获取[附属功能,以子应用查上级父应用,并验证权限]




子应用[3/4]
直接访问系统（系统进行两级授权：父级授权+验证父级是否有权限）
间接访问授权:[需暴露子应用的secret到外部,以父应用+子应用标识来查子应用serret]
	可以设置oauth登陆外部系统[父应用可访问子应用oauth secret]
	访问外部系统->根据访问数据系统得到key,完成验证-完成后续 [父应用可访问子应用secret]
