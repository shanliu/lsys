### 权限模块

> RBAC权限模块实现 逻辑跟验证权限过程参见以下说明

#### 权限中角色-用户-资源关系

> 资源属性: 资源操作 资源属于用户 资源句柄 例:
```
用户文章 
    资源操作:文章修改,文章新增,文章删除
    资源属于用户:文章用户ID
    资源句柄:文章ID+自定义字符生成
```

> 角色 用于关联用户跟资源 
```
用户:可以是指定用户ID,或全部用户,或已登录用户 [即:用户范围]
资源:可以是上面定义的,或任意资源,或禁止访问资源 [即:权限范围]
```
> 资源 - 角色 - 用户 ER 图
```mermaid
erDiagram
    RbacResModel ||--|{ RbacResOpModel : "资源 关联 资源操作"
    RbacResModel{
        int id PK "资源ID,可以当做一批待需授权操作的组"
        int user_id "用户ID"
        string res_key "资源句柄,用在代码中 可包含变量,如文章ID等"
        bool status "是否启用"
    }
    RbacResOpModel{
        int id PK "资源可执行操作ID"
        string name "资源操作名称,如查看,编辑,固定 不包含变量"
        string op_key "操作句柄,用在代码中"
        bool status "是否启用"
    }
    RbacRoleModel ||--o{ RbacRoleUserModel : "角色指定 有权限的访问用户"
    RbacRoleModel ||--o{ RbacRoleOpModel : "角色中包 含有 权限的访问资源"
    RbacRoleOpModel ||--|| RbacResOpModel : "角色中的访问资源 跟 资源操作数据关联"
    RbacRoleModel{
        int id PK "角色ID"
        int user_id "角色属于用户 0 表示系统角色"
        string name "角色名称"
        string relation_key "关系角色,用在代码中"
        int priority "角色优先级,查询授权时发现多个角色通过优先级判断使用哪个"
        int user_range "角色用户范围 包含:所有用户 登录用户 指定用户[RbacRoleUserRange::User] 指定关系用户[RbacRoleUserRange::Relation] 其中: 指定用户 时由 RbacRoleUserModel 决定 指定关系用户 时 编码时通过relation_key的传入决定"
        int res_op_range "角色授权访问资源范围 包含:可访问所有资源(AllowAll) 禁止访问所有资源(DenyAll) 可访问当前角色用户(AllowSelf)[资源user_id等于当前角色user_id]的资源 通过表关系定义的资源(AllowCustom)"
        bool status "是否启用"
    }
    RbacRoleOpModel{
        int role_id "角色ID -> RbacRoleModel ID"
        int res_op_id "资源可执行操作ID -> RbacResOpModel ID"
        bool positivity "存在值时,是授权操作[1]还是禁止操作[0]"
    }
    RbacRoleUserModel{
        int role_id "角色ID -> RbacRoleModel ID"
        int user_id "用户ID"
        bool status "是否启用"
        int timeout "授权有效期,可以用在付费订阅用户等有有效期的授权"
    }
    RbacRoleModel ||--o{ RbacTagsModel :"角色标签,可以当角色组或分类用"
    RbacResModel ||--o{ RbacTagsModel :"资源标签,可以当资源组或分类用"
    RbacTagsModel{
        int from_source "类型 用于区分 角色还是资源"
        int from_id "角色ID或资源ID"
        string name "标签名称"
        bool status "是否启用"
    }
```

#### 权限校验代码入口

> 参见 RbacAccess 的 check 方法,参数如下:

> 由 user_id 跟 relation_key_roles 作为 用户范围 来源 
```
参数作用说明:
通过 user_id 可以获取 公开用户 登陆用户 或 RbacRoleUserModel 3个途径的角色数据
通过 relation_key_roles 可以获取 特定关系 的角色
    当 relation_key_roles 中的 user_id = 0 时为系统控制的某些角色 如某subapp是否可以发送短信
    当 relation_key_roles 中的 user_id > 0 时为用户控制的某些角色，如某用户的粉丝，好友等
```

> 由 check_vec 作为 权限范围 其中一个来源
```
参数作用说明:
check_vec 由 RbacRoleOpModel 配置
RbacRoleResOpRange::AllowAll｜AllowSelf｜DenyAll 由代码定义特定逻辑
```

> 由上面的数据确定最后授权结果,如下图:


```mermaid
flowchart TB
    subgraph 用户范围[用户范围:指定被授权对象的范围]
    subgraph "全部用户 RbacRoleUserRange::Guest"
    subgraph "登录用户 RbacRoleUserRange::Login"
    end
    subgraph "指定用户 RbacRoleUserRange::User" 
    end
    subgraph "关系角色 RbacRoleUserRange::Relation" 
    end
    end
    end
    subgraph 权限范围[权限范围:可以被进行授权操作的范围]
    subgraph "全部可访问 RbacRoleResOpRange::AllowAll"
    subgraph "等于角色用户的权限 RbacRoleResOpRange::AllowSelf"
    end
    subgraph "由RbacRoleOpModel决定的权限 RbacRoleResOpRange::AllowCustom" 
    end
    end
    subgraph "全部禁止访问 RbacRoleResOpRange::DenyAll"
    end
    end
    用户范围 --> 角色[通过被授权对象范围 加 被授权操作范围 查询出符合条件的角色]
    权限范围 --> 角色 
    角色 --> 查询角色可以操作的资源 --> 排序角色并对跟请求的需操作的资源 --> 得出授权结果
```

#### 完整授权流程

```mermaid
stateDiagram-v2
    开始检查授权 --> 查询授权资源:根据请求数据中的资源参数查询
    查询授权资源 --> 授权结果:无需任何需验证资源，验证通过
    查询授权资源 --> 有未被管理的资源:存在未在数据库中但须验证资源
    有未被管理的资源 --> 授权结果:内置角色[编码实现]还存在记录，校验失败，否则验证通过
    查询授权资源 -->  需验证授权资源:需验证的资源都查找到数据库记录
    需验证授权资源 --> 查询角色:根据访问用户和资源得到角色
    查询角色 --> 查询内置角色:根据访问资源查询内置角色[编码实现]
    查询内置角色 --> 查询配置角色:查询数据库存储角色
    查询配置角色 --> 已登陆用户
    查询配置角色 --> 未登陆用户
    未登陆用户 --> 公开角色
    未登陆用户 --> 请求数据中指定角色名的角色
    已登陆用户 --> 公开角色
    已登陆用户 --> 请求数据中指定角色名的角色
    已登陆用户 --> 登陆用户公开角色
    已登陆用户 --> 已登陆用户的特定角色
    公开角色 --> 根据优先级合并角色
    登陆用户公开角色 --> 根据优先级合并角色
    请求数据中指定角色名的角色 --> 根据优先级合并角色
    已登陆用户的特定角色 --> 根据优先级合并角色
    根据优先级合并角色 --> 授权结果:根据合并后角色信息确定是否得到授权
```


#### 授权使用示例

> 确定资源类型

1. 系统资源 
```
用户id=0 
例: 用户能否进行登陆操作 应用能否发送短信等
```
2. 用户资源
```
用户id>0 
例: 用户发表文章能否被另一个用户查看 oauth用户时登陆信息被应用获取等
```

> 确定用户范围

1. 未登陆用户
2. 已登陆用户


```
例1 子应用的发送短信权限
1. 确定该操作为:用户对系统的操作
2. 预期该功能入参: 用户 应用id 短信内容 手机号
3. 定义资源：子应用 操作: 发送短信
4. 定义关系key：app-{应用id} 用户在系统总后台配置 该关系的权限
验证流程:由应用用户id+关系key+资源&发送操作 得出是否可以发送短信
```

```
例2 a用户查看b用户的某文章,a为b的粉丝
1. 确定该操作为:用户对用户的操作
2. 预期该功能入参: a用户id b用户id a用户跟b用户关系
3. 定义资源：{用户b}资源 操作: {文章id}查看
4. 定义关系key：firend-{b用户id} 用户b在用户后台或接口配置 该关系的权限
验证流程:由应用a用户id+关系key+{用户b}资源&{文章id}查看 得出是否可以发送短信
```