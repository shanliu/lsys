
##### web 应用集合模块

> 把各独立模块在这里集合并统一对外提供调用

> 功能实现不依赖具体的web框架,可自行选择web框架


用户 APP 待实现逻辑

RBAC.app_id=0

	不能放入REST接口
	//user_id=0  为系统
	//  资源&角色 为系统(后台增删改查)
	//  访问用户 限定 内部用户 app_id=0

	//user_id>0 用户系统权限,不区分是否内外部用户
	//  资源&角色 为内置(后台增删改查)
	//系统内用户资源控制 APPID =0

RBAC.app_id>0
	
	parent_app环境:
	system 0->u1

	u1 app1 
	u1 app2

	app1 u1->su1
	app1 u1->su2

	papp 环境下APP下REST使用 
		LOGIN:u1->su1 外部登录,通过PAPP完成
		
		RBAC: user 使用 user_data,不能用user_id
		res u1 app1;role u1 app1,role-user:u1->su*
		res u1->su1 app1;role u1->su1 app1,role-user:u1->su* papp可以REST编辑子用户权限
		check : user_data=>u1->su*

	sub_app环境:不支持