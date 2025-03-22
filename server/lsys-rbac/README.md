## 当前系统权限说明

> 本系统功能描述：授权方`系统或指定用户` 对 访问方`被授权用户或游客` 在进行 指定资源(系统资源或用户资源)访问时 进行授权的功能

> 本系统基于 [rbac模型](https://www.redhat.com/zh/topics/security/what-is-role-based-access-control) ,其中`user_id=0`的为系统权限,`user_id>0`的指定用户权限

> 资源约定:其中`user_id=0`的为系统资源,`user_id>0`的指定用户资源

> 限制:当角色`user_id=0`时，可往角色添加用户资源`user_id>0`,当角色`user_id>0`时,仅可添加用户跟角色`user_id`相同的资源

> 本系统在rbac模型的进行了以下扩充:

1. 在角色-用户关系

    - 字段为 user_range=RbacRoleUserRange::Custom,表示关联的用户由表`rbac_role_user`决定
    - 字段为 user_range=RbacRoleUserRange::Session,该角色不关联具体用户,用于授权会话时,由`role_key` 决定

    > RbacRoleUserRange::Session 使用场景说明:
    ```
    希望角色包含用户，独立于用户体系之外。例如:希望有一类角色在`编码阶段`定义时 , 如以下例子.
    1. 给全局或已登录用户赋予某种特殊权限,校验权限时多传入特定角色KEY.
    2. 访问者ip为某值时，希望有特殊的权限，则在代码中判断ip等于某值时，传入特定关系角色，以实现授权或禁止访问
    3. 当前系统的`应用`，目前以一个整体进行授权，所以在应用调用时会传入app-{id}的系统关系角色，再通过配置该角色来实现授权管理
    4. 当用户希望他的粉丝才能访问他的某些信息时，可以传入一个属于他的粉丝的关系，以实现授权或禁止访问
    总结：需要一个由系统外决定的角色时，可以使用`会话角色`来实现。
    ```

2. 在角色-功能关系

    a. 当角色 `user_id=0`时:

     - 字段为 res_range=RbacRoleResRange::Exclude,不可访问资源由表`rbac_perm`决定
     - 字段为 res_range=RbacRoleResRange::Any,可以访问任意资源
     - 字段为 res_range=RbacRoleResRange::Include,可以访问资源由表`rbac_perm`决定

    b. 当角色 `user_id>0`时:

      - 字段为 res_range=RbacRoleResRange::Exclude,不可访问资源由表`rbac_perm`决定,其中资源`user_id`必须角色`user_id`相同
      - 字段为 res_range=RbacRoleResRange::Any,可以访问该用户(ROLE.USER_ID)的任意资源
      - 字段为 res_range=RbacRoleResRange::Include,可以访问资源由表`rbac_perm`决定,其中资源`user_id`必须角色`user_id`相同

    > 约定优先级: `user_id=0` > `user_id>0` ; Exclude > Any > Include 

### 资源 - 角色 - 用户 ER 图
```mermaid
erDiagram
    RbacRoleModel{
        int id PK "角色ID"
        int user_id "0 表示系统角色 >0 为用户角色,且只能管理USER_ID等于该值的资源"
        string role_key "角色KEY,当user_range为标记用户作为标识:可理解为凭票通行,不管具体用户.role_key+user_id唯一"
        int user_range "角色用户范围 包含:指定用户,访问用户是否在(RbacRoleUserModel)中(Custom); 标记用户,访问用户携带标记,由role_key标识并配置具体功能(Exter)"
        int res_range "角色授权访问资源范围 包含:通过表(RbacPermModel)关系定义禁止的资源(Exclude) 通过表(RbacPermModel)关系定义授权的资源(Include) 访问任意资源(Any)"
        string role_name "角色名称"
        bool status "是否启用"
    }
    RbacResModel{
        int id PK "资源ID,可以当做一批待需授权操作的组"
        string res_type "资源类型"
        int user_id "用户ID res跟op关联条件 res.user_id=op.user_id"
        string res_data "资源附带数据,如文章ID等"
        string res_name "资源名称"
        bool status "是否启用"
    }
    RbacOpModel{
        int id PK "操作ID,资源可用操作"
        int user_id "用户ID"
        string op_key "资源操作KEY"
        string op_name "资源名称"
        bool status "是否启用"
    }
    RbacOpResModel{
        string res_type "资源类型 role.user_id=op.user_id"
        int op_id "资源操作ID"
        bool status "是否启用"
    }
    RbacPermModel{
        int role_id "角色ID,role.user_id>0时,role.user_id=res.user_id"
        int res_id "资源ID res.user_id=op.user_id"
        int op_id "资源操作ID"
        bool status "是否启用"
    }
    RbacRoleUserModel{
        int role_id "角色ID -> RbacRoleModel ID"
        int user_id "用户ID"
        int timeout "授权有效期,可以用在付费订阅用户等有有效期的授权"
        bool status "是否启用"
    }
    RbacOpModel ||--o{ RbacOpResModel :"操作 跟 资源分类 res_type 关系"
    RbacRoleModel ||--o{ RbacPermModel : "角色 跟 权限关系"
    RbacResModel ||--o{ RbacPermModel : "资源 跟 权限关系"
    RbacOpModel ||--o{ RbacPermModel : "操作 跟 权限关系"
    RbacRoleModel ||--o{ RbacRoleUserModel : "角色 跟 用户 关系"
    RbacResModel ||--o{ RbacOpResModel :"资源分类 res_type 跟 操作关系"
```


### 授权流程

> 当访问的`user_id=0`时表示游客,大于0时为指定用户ID进行授权检查

> 资源由资源类型+资源数据组成,如:资源类型:文章 资源数据:文章ID

> `超级用户`跟`用户自身资源`在代码中处理.双向反查时[指定用户获取可访问资源,指定资源获取可访问用户]也同样做特殊处理.

> 鉴权时先根据访问的信息,查询出可用的角色,在进行鉴权处理

```mermaid
stateDiagram-v2
    [*]  --> 收集访问数据:收集 用户属性[用户ID(游客)] 资源属性[访问资源列表] 环境属性[来源IP等]

    收集访问数据 --> 已配置资源:根据访问数据查询已配置资源跟操作数据
    收集访问数据 --> 已配置角色:根据 已配置资源 会话角色 访问用户 得到所有相关角色并合并

    已配置资源 --> 资源角色校验:根据访问数据,已配置资源,已配置角色进入权限校验流程
    已配置角色 --> 资源角色校验:根据访问数据,已配置资源,已配置角色进入权限校验流程

    资源角色校验 --> 超级用户角色:如果访问用户ID在超级用户列表中
    超级用户角色 -->  [*] :是超级用户,通过授权验证,记录日志

    资源角色校验 --> 非超级用户

    非超级用户 --> 系统角色校验:根据资源 系统角色配置 进行权限校验(阻止访问,任意访问,指定访问)
    系统角色校验 --> 系统角色禁止授权:标记访问资源禁止访问,继续判断下一个访问资源
    系统角色校验 --> 系统角色已授权:标记访问资源已授权,继续判断下一个访问资源
    系统角色校验 --> 系统角色无授权:未在配置user_id=0授权
    系统角色无授权 --> 是自身资源:访问用户等于资源用户,标记访问资源已授权,继续判断下一个访问资源
    系统角色无授权 --> 非自身资源:访问用户不等于资源用户
    非自身资源 --> 用户角色校验:根据资源 用户角色配置 进行权限校验(阻止访问,任意访问,指定访问)
    用户角色校验 --> 用户角色禁止授权:标记访问资源禁止访问,继续判断下一个访问资源
    用户角色校验 --> 用户角色已授权:标记访问资源已授权,继续判断下一个访问资源
    用户角色校验 --> 用户角色无授权:标记访问资源禁止访问,继续判断下一个访问资源
    系统角色禁止授权 --> 合并授权结果:无下一个访问资源校验时,完成验证
    系统角色已授权 --> 合并授权结果:无下一个访问资源校验时,完成验证
    是自身资源 --> 合并授权结果:无下一个访问资源校验时,完成验证
    用户角色禁止授权 --> 合并授权结果 :无下一个访问资源校验时,完成验证
    用户角色已授权 --> 合并授权结果 :无下一个访问资源校验时,完成验证
    用户角色无授权 --> 合并授权结果:无下一个访问资源校验时,完成验证
    合并授权结果 -->  [*]:合并循环验证的结果,所有资源均授权时,通过授权验证,记录日志
```


### 使用系统参考:

> 以下为不同场景的配置参考:


1. 控制系统资源是否可被指定角色的用户访问

```
1. 通过接口或后台建立资源
2. 系统建立角色
    用户范围:指定用户(RbacRoleUserRange::Custom)
    授权范围:自定义配置访问资源(RbacRoleResRange::Include):关联上面建立资源
```

2. 控制系统或用户资源是否可被特定角色访问

> 系统资源示例： 特定IP可访问，当IP为指定IP时，权限校验传入特定角色参数

```
1. 通过接口或后台建立资源
2. 系统或用户建立角色
    用户范围:会话角色(RbacRoleUserRange::Session)
    授权范围:特定IP全局可访问时，可用 (RbacRoleResRange::Any) 操作 或 可访问某些权限 (RbacRoleResRange::Include)
```

> 用户资源示例：粉丝关系，当为粉丝关系时，权限校验传入特定角色参数

```
1. 已知数据: a用户id b用户id a用户跟b用户关系
2. 定义资源：资源: {用户b}{文章id} 操作: {文章id}查看
3. 用户b在用户后台或通过接口配置该关系的权限:定义关系key：{用户b}的firend
4. 访问时，用户a授权传入关系key:{用户b}的firend
5. 系统或用户建立角色
    用户范围:会话角色(RbacRoleUserRange::Session)
    授权范围:可访问某些权限 (RbacRoleResRange::Include)
```

3. 系统指定用户赋予所有权限

```
1. 系统建立角色
    用户范围:指定用户(RbacRoleUserRange::Custom)
    授权范围:授权访问任何资源(RbacRoleResRange::Any)
```


### 一些常见问题

1. 关于如何定义资源创建:
> 此操作可通过 系统后台 完成. 在`系统管理`-`授权管理`-`资源管理` 中操作
```
例如:文章[资源类型:文章 资源数据:文章ID 资源名称:文章标题]
```

2. 关于解决`会话角色`问题:
> 此操作可通过 系统后台 完成. 在 `用户中心`-`授权管理` 或 `系统管理`-`授权管理`-`系统角色` 中操作
```
例如:当B为A的粉丝时,B才能看A的文章
可以先定义一个`会话角色`: A的粉丝的角色
权限系统提供`A的粉丝的角色`的可用权限管理操作
校验权限时:根据查询B是否A粉丝,传入上面定义的`会话角色`的标识符,完成权限校验
```
```
例如:当为VIP级别为N时,可参与某活动
可以先定义一个`会话角色`: VIP为N的角色
权限系统提供`VIP为N的角色`的可用权限管理操作
校验权限时:根据用户的VIP等级传入对应的`VIP为N的角色`,完成权限校验
```

3. 关于校验权限时所需参数问题:
> 验证权限时，需要的参数预知以下参数:
```
1. 用户ID,游客传入时用户ID为0
2. 收集需要校验权限的 资源权限标识 参见(问题1,问题2)[check_vec]
3. 收集 会话角色 的标识符列表 参见(问题3)[role_key_vec]
4. 完成以上参数收集后，在通过校验权限接口完成权限校验 :参见 RbacAccess 的 check 方法
```
---