### 服务层实现

> 执行 sql_merge.[sh|cmd] 创建汇总各模块使用到的SQL语句:tables.sql 后导入到mysql中.

> 服务层依赖: mysql+redis 在 ./examples/lsys-actix-web/.env 配置

> 使用示例[examples]目前基于 actix-web 作为接入web框架，你可选择其他基于 tokio 的web框架

### 编译&&启动

1. 导入SQL到数据库
```
./sql_merge.[sh|cmd] #创建tables.sql
mysql -u your_username -p your_database < ./tables.sql #导入SQL到mysql
```

2. 修改你的REDIS,MYSQL,jwt token等配置
> [./examples/lsys-actix-web/.env 的配置会覆盖./examples/lsys-actix-web/config/下配置]

```
./examples/lsys-actix-web/.env 
```

3. 启动开发环境
```
cd ./examples/lsys-actix-web/ && cargo run #cargo run -r 
```



### 接入时序说明

#### 内置账号相关接入流程图

> 应用添加流程

```mermaid
sequenceDiagram
    actor LSYS_账号用户
    participant 外部系统
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access
    participant LSYS_账号  as lsys-user

    par LSYS_应用 添加
        LSYS_账号用户 ->> LSYS_系统 :访问
        LSYS_系统 -->> LSYS_账号:完成 LSYS_账号 校验
        LSYS_账号 -->> LSYS_授权:通过 lsys-access 设置登录信息
        LSYS_授权 -->> LSYS_应用:鉴权通过后,新增应用
    end
```

> 内部应用使用流程

```mermaid
sequenceDiagram
    actor  LSYS_账号用户
    
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access
    participant LSYS_账号  as lsys-user

    par 外部系统 获取 LSYS_应用 信息
       LSYS_账号用户 ->> LSYS_系统 :登录访问
       LSYS_系统 -->> LSYS_账号:系统  LSYS_账号 模块获取信息
       LSYS_账号 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_应用:查看并获取 应用_ID_秘钥 
       LSYS_应用 -) 外部系统:把 应用_ID_秘钥 写入外部系统
    end
    par 外部系统 访问 LSYS_应用 功能 (如外部用户接入)
        外部系统 ->> LSYS_系统: 以HTTP接口方式,使用 应用_ID_秘钥_签名方式 完成访问
        LSYS_系统 -->> LSYS_应用: 根据 应用_ID_签名 进行校验
        LSYS_应用 -->> LSYS_系统:通过后继续执行具体业务逻辑
        LSYS_系统 ->> LSYS_系统:完成具体业务流程执行
        LSYS_系统 ->> 外部系统:由LSYS_系统输出业务信息
    end
```

> 外部应用使用流程

```mermaid
sequenceDiagram
    actor  LSYS_账号用户A
    actor  LSYS_账号用户B
    participant 外部系统A
    participant 外部系统B
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access
    participant LSYS_账号  as lsys-user

    par 外部系统A 获取 LSYS_应用 信息
       LSYS_账号用户A ->> LSYS_系统 :登录访问
       LSYS_系统 ->> LSYS_账号:系统  LSYS_账号 模块获取信息
       LSYS_账号 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_应用:查看并获取 应用A_ID_秘钥
       LSYS_应用 -->> LSYS_应用:关联被访问用户
       LSYS_应用 -) 外部系统A:把 应用A_ID_秘钥 写入外部系统
    end
    par 外部系统B 获取 LSYS_应用 信息
       LSYS_账号用户B ->> LSYS_系统:登录访问
       LSYS_系统 -->> LSYS_账号:系统  LSYS_账号 模块获取信息
       LSYS_账号 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_应用:查看并获取 应用B_ID_关联秘钥
       LSYS_应用 -) 外部系统B:把 应用B_ID_关联秘钥 写入外部系统
    end
    par 外部系统B 在 外部系统A 使用 LSYS_应用 进行访问
        外部系统B ->> 外部系统A :以HTTP接口方式,使用 应用B_ID_关联秘钥_签名方式 访问
        外部系统A ->> LSYS_系统: 以HTTP接口方式,使用 应用A_ID_秘钥_签名方式 及 应用B_ID 完成访问
        LSYS_系统 -->> LSYS_应用: 根据 应用_ID_签名 进行校验
        LSYS_应用 -->> LSYS_应用 : 根据 应用B_ID 查询 应用B信息及关联秘钥
        LSYS_应用 -->> LSYS_系统:通过并完成查询后返回调用入口 
        LSYS_系统 ->> 外部系统A:由 LSYS_系统 输出  应用B信息及关联秘钥
        外部系统A ->> 外部系统A:根据 应用B信息及关联秘钥 及 应用B_ID 完成校验及执行业务
        外部系统A ->> 外部系统B:返回结果,外部系统B进行后续流程处理
    end

```

> OAUTH应用使用流程

```mermaid
sequenceDiagram
    actor  LSYS_账号用户
    participant 外部系统
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access
    participant LSYS_账号  as lsys-user

    par 外部系统获取 LSYS_应用 OAUTH信息
        LSYS_账号用户 ->> LSYS_系统 :登录访问
        LSYS_系统 ->> LSYS_账号:系统  LSYS_账号 模块获取信息
        LSYS_账号 -->> LSYS_授权:通过 lsys-access 设置登录信息
        LSYS_授权 -->> LSYS_应用:查看并获取 应用_ID_OAUTH秘钥
        LSYS_应用 -) 外部系统:把 应用_ID_OAUTH秘钥 写入外部系统
    end
    par 外部系统使用 LSYS_应用 进行OAUTH登录
        LSYS_账号用户 ->> 外部系统: 访问外部系统登录页面
        外部系统 ->> LSYS_系统:用 应用_ID 完成OAUTH跳转
        LSYS_系统 -->> LSYS_应用:根据 应用_ID 检查应用
        LSYS_应用 -->> LSYS_账号:使用 LSYS_账号 完成权限校验
        LSYS_账号 -->> LSYS_应用:由 LSYS_账号 提供信息,把登录code信息及scope数据保存 在 LSYS_应用缓存
        LSYS_应用 --> LSYS_授权:是否同时设置登录(可选)
        LSYS_应用 -->> LSYS_系统:返回调用入口
        LSYS_系统 ->> 外部系统:携带code并调回外部系统
        外部系统 ->>外部系统 :外部系统内部逻辑处理
        外部系统 ->> LSYS_系统:根据code及 应用_ID_OAUTH秘钥 访问
        LSYS_系统 -->> LSYS_应用:检查应用_ID_OAUTH秘钥检测是否有效,获取code对应信息,并生成 TOKEN
        LSYS_应用 -->> LSYS_授权:由 LSYS_应用 提交TOKEN及登录信息
        LSYS_授权 -->> LSYS_系统:返回调用入口
        LSYS_系统 ->> 外部系统:把TOKEN输出返回外部
        外部系统 ->>外部系统 :外部系统内部逻辑处理
        外部系统 ->> LSYS_系统:以 应用_ID_秘钥_签名_TOKEN 访问
        LSYS_系统 -->> LSYS_应用:根据 应用_ID_签名 进行校验
        LSYS_应用 -->> LSYS_授权:根据 提交TOKEN及登录 校验
        LSYS_授权 -->> LSYS_系统:通过后 执行具体业务逻辑
        LSYS_系统 ->> 外部系统:外部系统得到数据,继续完成其他业务逻辑
    end
```

#### 外部账号相关接入流程图

> 外部账号接入 依赖: 内置账号 > 内部应用流程 




> 内部应用使用流程

```mermaid
sequenceDiagram
    actor  外部系统A账号
    participant 外部系统A
    participant 外部系统B
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access

    par 外部系统A 获取 LSYS_应用 信息
       外部系统A账号 ->> 外部系统A : 进行外部账号验证,通过后继续
       外部系统A ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
       LSYS_系统 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_系统:返回接口入口
       LSYS_系统 ->> 外部系统A:成功后,准备下一步跳转到 LSYS_系统
       外部系统A ->> LSYS_系统 :以表单POST跳转登录 LSYS_系统 后台 
       LSYS_系统 -->> LSYS_应用:查看并获取 应用_ID_秘钥 
       LSYS_应用 -) 外部系统A:把 应用_ID_秘钥 写入外部系统
    end
    par 外部系统B 访问 LSYS_应用 功能 (如外部用户接入)
        外部系统B ->> LSYS_系统: 以HTTP接口方式,使用 应用_ID_秘钥_签名方式 完成访问
        LSYS_系统 -->> LSYS_应用: 根据 应用_ID_签名 进行校验
        LSYS_应用 -->> LSYS_系统:通过后继续执行具体业务逻辑
        LSYS_系统 ->> LSYS_系统:完成具体业务流程执行
        LSYS_系统 ->> 外部系统B:由LSYS_系统输出业务信息
    end
```

> 外部账号 登录 LSYS_系统 

```mermaid
sequenceDiagram
    actor  外部账号
    participant 外部系统
    participant LSYS_系统
    participant 外部系统_LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access

    par 外部系统 获取 LSYS_应用 信息
       外部账号 ->> 外部系统 : 进行外部账号验证,通过后继续
       外部系统 ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
       LSYS_系统 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_系统:返回接口入口
       LSYS_系统 ->> 外部系统:成功后,准备下一步跳转到 LSYS_系统
       外部系统 ->> LSYS_系统 :以表单POST跳转登录 LSYS_系统 后台 
       LSYS_系统 -->> LSYS_应用:查看并获取 应用_ID_秘钥 
       LSYS_应用 -) 外部系统:把 应用_ID_秘钥 写入外部系统
    end

    par 外部系统 设置 LSYS_系统 登录信息
        外部账号 ->> 外部系统 : 进行外部账号验证,通过后继续
        外部系统 ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
        LSYS_系统 -->> 外部系统_LSYS_应用: 根据 应用_ID_签名 进行校验
        外部系统_LSYS_应用 -->> LSYS_系统:通过后继续执行具体业务逻辑
        LSYS_系统 -->> LSYS_授权:由外部系统提交登录信息保存
        LSYS_授权 -->> LSYS_系统:完成登录逻辑
        LSYS_系统 ->> 外部系统:返回登录结果,完成本地登录逻辑
        外部系统  ->> LSYS_系统:可以 以表单POST跳转登录 LSYS_系统 后台 (可选)
        外部系统  ->> 外部账号:返回结果给用户
    end
    par 外部系统 检查 LSYS_系统 是否登录指定外部账号
        外部系统 ->> LSYS_系统: 以HTTP接口方式 使用 TOKEN 检测是否登录
        LSYS_系统 -->> 外部系统_LSYS_应用: 根据 应用_ID_签名 进行校验
        外部系统_LSYS_应用 -->> LSYS_系统:通过后继续执行具体业务逻辑
        LSYS_系统 -->> LSYS_授权:由外部系统提交TOKEN检测是否存在
        LSYS_授权 -->> LSYS_系统:完成登录逻辑判断,存在登录返回登录信息
        LSYS_系统 ->> 外部系统:返回登录结果
        外部系统  ->> 外部账号:返回结果给用户
    end
```


> 外部应用使用流程

```mermaid
sequenceDiagram
    actor  外部系统A账号A
    actor  外部系统A账号B
    participant 外部系统A
    participant 外部系统B
    participant 外部系统C
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access

    par 外部系统B 获取 LSYS_应用 信息
       外部系统A账号A ->> 外部系统A : 进行外部账号验证,通过后继续
       外部系统A ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
       LSYS_系统 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_系统:返回接口入口
       LSYS_系统 ->> 外部系统A:成功后,准备下一步跳转到 LSYS_系统
       外部系统A ->> LSYS_系统 :以表单POST跳转登录 LSYS_系统 后台 
       LSYS_系统 -->> LSYS_应用:查看并获取 应用_ID_秘钥 
       LSYS_应用 -) 外部系统B:把 应用B_ID_秘钥 写入外部系统
    end
    par 外部系统C 获取 LSYS_应用 信息
       外部系统A账号B ->> 外部系统A : 进行外部账号验证,通过后继续
       外部系统A ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
       LSYS_系统 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_系统:返回接口入口
       LSYS_系统 ->> 外部系统A:成功后,准备下一步跳转到 LSYS_系统
       外部系统A ->> LSYS_系统 :以表单POST跳转登录 LSYS_系统 后台 
       LSYS_系统 -->> LSYS_应用:查看并获取 应用_ID_秘钥 
       LSYS_应用 -) 外部系统C:把 应用C_ID_秘钥 写入外部系统
    end
    par 外部系统B 在 外部系统C 使用 LSYS_应用 进行访问
        外部系统B ->> 外部系统C :以HTTP接口方式,使用 应用C_ID_关联秘钥_签名方式 访问
        外部系统C ->> LSYS_系统: 以HTTP接口方式,使用 应用B_ID_秘钥_签名方式 及 应用C_ID 完成访问
        LSYS_系统 -->> LSYS_应用: 根据 应用_ID_签名 进行校验
        LSYS_应用 -->> LSYS_应用 : 根据 应用C_ID 查询 应用B信息及关联秘钥
        LSYS_应用 -->> LSYS_系统:通过并完成查询后返回调用入口 
        LSYS_系统 ->> 外部系统C:由 LSYS_系统 输出  应用B信息及关联秘钥
        外部系统C ->> 外部系统C:根据 应用C信息及关联秘钥 及 应用C_ID 完成校验及执行业务
        外部系统C ->> 外部系统B:返回结果,外部系统C进行后续流程处理
    end

```


> OAUTH应用使用流程

```mermaid
sequenceDiagram
    actor  外部系统A账号A
    actor  外部系统A账号B
    actor  外部系统A账号C
    participant 外部系统A
    participant 外部系统B
    participant LSYS_系统
    participant LSYS_应用 as lsys-app
    participant LSYS_授权 as lsys-access

    par 外部系统A 获取 LSYS_应用 信息
       外部系统A账号A ->> 外部系统A : 进行外部账号验证,通过后继续
       外部系统A ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
       LSYS_系统 -->> LSYS_授权:通过 lsys-access 设置登录信息
       LSYS_授权 -->> LSYS_系统:返回接口入口
       LSYS_系统 ->> 外部系统A:成功后,准备下一步跳转到 LSYS_系统
       外部系统A ->> LSYS_系统 :以表单POST跳转登录 LSYS_系统 后台 
       LSYS_系统 -->> LSYS_应用:查看并获取 应用A_ID_秘钥 
       LSYS_应用 -) 外部系统A:把 应用A_ID_秘钥 写入外部系统
    end
    par 通过 外部系统A 获取 外部系统B 需要的 LSYS_应用 OAUTH信息
        外部系统A账号B ->> 外部系统A : 进行外部账号验证,通过后继续
        外部系统A ->> LSYS_系统: 以HTTP接口方式 使用 当前用户信息及TOKEN 完成登录
        LSYS_系统 -->> LSYS_授权:通过 lsys-access 设置登录信息
        LSYS_授权 -->> LSYS_系统:返回接口入口
        LSYS_系统 ->> 外部系统A:成功后,准备下一步跳转到 LSYS_系统
        外部系统A ->> LSYS_系统 :以表单POST跳转登录 LSYS_系统 后台 
        LSYS_系统 -->> LSYS_应用:查看并获取 应用B_ID_OAUTH秘钥
        LSYS_应用 -->> 外部系统B:把 应用B_ID_OAUTH秘钥 写入 外部系统B
    end
    
    par 外部系统使用 LSYS_应用 进行OAUTH登录
        外部系统A账号C ->> 外部系统B: 访问外部系统登录页面
        外部系统B ->> 外部系统A:用 应用B_ID 完成OAUTH跳转
        外部系统A ->> LSYS_系统:以HTTP接口方式 使用 应用A_ID_秘钥 及 应用B_ID 查询应用信息
        LSYS_系统 -->> LSYS_应用:校验 应用A 成功后,根据 应用B_ID 检查应用
        LSYS_应用 -->> LSYS_系统:完成查下返回调用入口
        LSYS_系统 ->> 外部系统A:符合 应用B 在应用A 下.使用 外部系统A账号C 完成登录校验
        外部系统A  ->> LSYS_系统:以HTTP接口方式 使用 应用A_ID_秘钥 及 登录code信息及scope数据 保存CODE信息
        LSYS_系统 -->> LSYS_应用:校验 应用A 成功后,根据 应用B_ID 对应CODE的登录信息, 在 LSYS_应用缓存
        LSYS_应用 -->> LSYS_系统:完成查下返回调用入口
        LSYS_系统 ->> 外部系统A:完成登录code相关设置,返回 外部系统B
        外部系统A ->> 外部系统B:携带code并调回 外部系统B
        外部系统B ->>外部系统B :外部系统B 内部逻辑处理
        外部系统B ->> 外部系统A:根据code及 应用B_ID_OAUTH秘钥 访问
        外部系统A ->> LSYS_系统:以HTTP接口方式 使用 code及 应用B_ID_OAUTH秘钥 完成OAUTH
        LSYS_系统 -->> LSYS_应用:校验 应用A 成功后,根据 应用B_ID 检查应用
        LSYS_应用 -->> LSYS_应用:检查 应用B_ID_OAUTH秘钥 是否有效,获取code对应信息,并生成 TOKEN
        LSYS_应用 -->> LSYS_授权:由 LSYS_应用 提交TOKEN及登录信息
        LSYS_授权 -->> LSYS_系统:返回调用入口
        LSYS_系统 ->> 外部系统A:成功得到TOKEN
        外部系统A ->> 外部系统B:把TOKEN输出返回
        外部系统B ->>外部系统B :外部系统B 内部逻辑处理
        外部系统B ->> 外部系统A: 以 应用B_ID_秘钥_签名_TOKEN 访问
        外部系统A ->> LSYS_系统:以  应用A_ID_秘钥 及 应用B_ID_秘钥_签名_TOKEN 访问
        LSYS_系统 -->> LSYS_应用:校验 应用A 信息及应用B关系
        LSYS_应用 -->> LSYS_授权:校验 应用B_TOKEN 
        LSYS_授权 -->> LSYS_系统:成功后返回调用入口,返回登录信息
        LSYS_系统 ->> 外部系统A: 获取到 登录信息 完成其他业务逻辑
        外部系统A ->> 外部系统B:继续 外部系统B 其他业务逻辑
    end
```



#### 相关节点说明

1. LSYS_授权 : 完成登录用户鉴权及用户映射系统

2. LSYS_账号 : 为内置的账号系统,完成内部账号登录依赖 LSYS_授权

3. LSYS_应用 : 应用管理系统,依赖 LSYS_授权 前置鉴权 

4. LSYS_系统 : 整体系统用户依赖 LSYS_授权 , LSYS_账号 为可选内置用户系统

5. 外部账号 : 账号相关信息由外部独立系统实现

6. 外部系统_LSYS_应用 : 外部系统对应在 LSYS_系统 的一个或多个 LSYS_应用 

7. 外部系统 : 外部账号登录外部系统完成,可 LSYS_应用 方式接入 LSYS_系统 ,并由 LSYS_授权 完成到 LSYS_系统 登录
