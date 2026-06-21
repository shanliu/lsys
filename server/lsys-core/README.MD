# lsys-core

lsys-core 是一个 web 基础库，封装了常用的基础功能模块，用于快速构建web应用。

## 主要功能

- **统一配置 `config`** - 统一规则处理
- **参数校验** - 参数验证框架，支持多种内置规则
- **国际化支持** - 基于 Fluent 的多语言支持
- **数据库操作** - 基于 SQLx 的数据库抽象层，支持 MySQL、PostgreSQL、SQLite,完全使用数据绑定
- **缓存管理** - 多节点同步清理的本地内存缓存实现
- **远程任务 `listen_notify`** - 基于 Redis,将任务派发到多个监听节点中的一个去执行并返回
- **分布式任务 `task_dispatch`** - 基于 Redis,将任务分拆到多个主机上分别执行
- **多节点同步 `remote_notify`** - 基于 Redis, 将消息通知到所有节点
- **分布式任务锁 `timeout_task`** - 基于 Redis, 多个节点下同个任务,仅有一个节点执行任务处理
- **验证码处理** - 验证码生成、存储、校验的完整解决方案
- **统一输入输出** - 通过输入输出格式规范跟一些常用辅助函数(可选)


## 功能特性

### 可选特性 (Features)

```toml
[dependencies]
lsys-core = { version = "0.0.1", features = ["db-mysql", "redis", "tracing", "tera"] }
```

- `tokio` - 启用异步运行时支持
- `tracing` - 启用日志追踪功能
- `db` - 启用数据库基础功能
- `db-mysql` - 启用 MySQL 数据库支持
- `db-postgres` - 启用 PostgreSQL 数据库支持
- `db-sqlite` - 启用 SQLite 数据库支持
- `redis` - 启用 Redis 缓存、任务派发、远程通知等功能
- `tera` - 启用模板引擎支持

## 快速开始

### 1. 应用初始化与配置管理

```rust
use lsys_core::app_core::AppCore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化应用核心
    // AppCore 会自动加载配置文件
    let app_core = Arc::new(
        AppCore::new(
            "./",              // 应用目录
            "config",          // 配置目录
            "app.toml",        // 主配置文件
            Some(&["db.toml", "redis.toml"]), // 其他配置文件，支持多个，如为每个 CRATE 定义一个 CONFIG
        ).await?
    );
    
    // 读取主配置文件的配置项
    let db_host = app_core.config.find(None).get_string("database.host")?;
    let redis_url = app_core.config.find(None).get_string("redis.url")?;
    let max_connections = app_core.config.find(None).get_int("database.max_connections")?;
    
    // 读取特定 crate 的配置（如果在初始化时指定了多个配置文件）
    // 例如读取 db.toml 中的配置
    let pool_size = app_core.config.find(Some("db")).get_int("pool_size")?;
    
    // 获取配置文件中的路径（自动处理相对路径转换为绝对路径）
    let log_path = app_core.config_path(
        app_core.config.find(None),
        "log.path"
    )?;
    
    println!("数据库主机: {}", db_host);
    println!("Redis URL: {}", redis_url);
    println!("日志路径: {:?}", log_path);
    
    Ok(())
}
```

### 2. 数据库操作

#### 基础查询构建

```rust
use lsys_core::db::{QueryBuilderExt, TableName, Field};
use sqlx::{MySql, QueryBuilder};

// 创建查询构建器
let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM ");
qb.push(TableName::new("users").full_name());

// 添加 WHERE 条件
qb.push_where()
    .field_eq("status", 1)
    .push_and()
    .field_in_copied("role_id", &[1, 2, 3])
    .push_and()
    .field_like("username", "%admin%");

// 执行查询
let users = qb.build_query_as::<User>()
    .fetch_all(&pool)
    .await?;
```

#### INSERT 操作

```rust
use lsys_core::db::{Insert, Field};

// 假设已定义 UserModel 实现了 TableMeta trait
// Insert 需要指定数据库类型和表模型
let result = Insert::<_, UserModel>::new()
    .set(UserModel::USERNAME, "admin")
    .set(UserModel::EMAIL, "admin@example.com")
    .set(UserModel::STATUS, 1)
    .execute(&pool)
    .await?;

// 获取插入的 ID
let user_id = result.last_insert_id();
```

#### 批量 INSERT

```rust
use lsys_core::db::BatchInsert;

let batch = BatchInsert::<_, UserModel>::new()
    .push(
        Insert::<_, UserModel>::new()
            .set(UserModel::USERNAME, "user1")
            .set(UserModel::EMAIL, "user1@example.com")
    )
    .push(
        Insert::<_, UserModel>::new()
            .set(UserModel::USERNAME, "user2")
            .set(UserModel::EMAIL, "user2@example.com")
    );

let result = batch.execute(&pool).await?;
```

#### INSERT ON DUPLICATE KEY UPDATE (MySQL)

```rust
use lsys_core::db::{Insert, Update};

let insert = Insert::<_, UserModel>::new()
    .set(UserModel::USERNAME, "admin")
    .set(UserModel::EMAIL, "admin@example.com");

let update = Update::<_, UserModel>::new()
    .set(UserModel::EMAIL, "newemail@example.com")
    .set(UserModel::UPDATED_AT, now_time());

let result = insert.execute_update(update, &pool).await?;
```

#### UPDATE 操作

```rust
use lsys_core::db::{Update, QueryBuilderExt};

// 使用 Update 构建器并执行
let result = Update::<_, UserModel>::new()
    .set(UserModel::STATUS, 2)
    .set(UserModel::UPDATED_AT, now_time())
    .execute(&pool, |qb| {
        qb.push_where().field_eq("id", user_id);
    })
    .await?;

// 更复杂的 WHERE 条件
let result = Update::<_, UserModel>::new()
    .set(UserModel::STATUS, 1)
    .execute(&pool, |qb| {
        qb.push_where()
            .field_eq("status", 0)
            .push_and()
            .field_lt("created_at", expired_time);
    })
    .await?;
```

### 3. 缓存管理

```rust
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::remote_notify::RemoteNotify;
use std::sync::Arc;

// 配置缓存
let cache_config = LocalCacheConfig::new(
    "user_cache",      // 缓存名称
    Some(1000),        // 最大缓存数量
    Some(300),         // 缓存时间（秒）
);

// 创建缓存实例（通常在 DAO 结构体中）
let cache = Arc::new(LocalCache::new(
    remote_notify,     // Arc<RemoteNotify>
    cache_config,
));

// 获取缓存
if let Some(user) = cache.get(&user_id).await {
    return Ok(user);
}

// 设置缓存
cache.set(user_id, user.clone(), 0).await;

// 清除缓存（会通知其他所有节点完成清理）
cache.clear(&user_id).await;

// 自动回填缓存
let user = cache.get_or_fetch(&user_id, || async {
    // 从数据库获取数据
    fetch_user_from_db(user_id).await
}).await?;
```

### 4. 参数校验

```rust
use lsys_core::valid_param::{
    ValidParam, ValidParamCheck, ValidEmail, ValidStrlen, ValidRange
};
use lsys_core::valid_key;

// 定义校验规则
let result = ValidParam::default()
    .add(
        valid_key!("email"),
        &email,
        &ValidParamCheck::default()
            .add_rule(ValidEmail)
            .add_rule(ValidStrlen::max(100))
    )
    .add(
        valid_key!("age"),
        &age,
        &ValidParamCheck::default()
            .add_rule(ValidRange::new(18, 120))
    )
    .check();

if let Err(e) = result {
    // 处理校验错误
    println!("验证失败: {:?}", e);
}
```

#### 内置校验规则 `可自行实现trait`

- `ValidEmail` - 邮箱格式
- `ValidStrlen` - 字符串长度
- `ValidRange` - 数值范围
- `ValidPattern` - 正则表达式
- `ValidChinaID` - 中国身份证号
- `ValidChinaLicensePlate` - 中国车牌号
- `ValidCreditCard` - 信用卡号
- `ValidDomain` - 域名格式
- `ValidColor` - 颜色格式（HEX/RGB）
- `ValidDateTime` - 日期时间格式

### 5. 验证码处理

```rust
use lsys_core::valid_code::{ValidCode, ValidCodeDataRandom, CheckCodeData};

// 创建验证码管理器（通常在 DAO 中作为方法）
fn valid_code(redis: deadpool_redis::Pool) -> ValidCode {
    ValidCode::new(
        redis,
        "email",           // 前缀
        true,              // 忽略大小写
        Some(6),           // 最大尝试次数
    )
}

// 创建验证码构造器
fn valid_code_builder() -> ValidCodeDataRandom {
    ValidCodeDataRandom::new(
        300,               // 保存时间（秒）
        60,                // 重复生成间隔（秒）
    )
}

// 生成验证码
async fn set_valid_code(
    redis: deadpool_redis::Pool,
    account_id: u64,
    email: &str,
) -> Result<(String, usize), Box<dyn std::error::Error>> {
    let mut builder = valid_code_builder();
    let tag = format!("{}-{}", account_id, email);
    
    let (code, ttl) = valid_code(redis)
        .set_code(&tag, &mut builder)
        .await?;
    
    println!("验证码: {}, 有效期: {}秒", code, ttl);
    Ok((code, ttl))
}

// 验证验证码
async fn check_valid_code(
    redis: deadpool_redis::Pool,
    code: &str,
    account_id: u64,
    email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tag = format!("{}-{}", account_id, email);
    
    valid_code(redis)
        .check_code(&CheckCodeData::new(&tag, code))
        .await?;
    
    println!("验证成功");
    Ok(())
}

// 清除验证码
async fn clear_valid_code(
    redis: deadpool_redis::Pool,
    account_id: u64,
    email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = valid_code_builder();
    let tag = format!("{}-{}", account_id, email);
    
    valid_code(redis)
        .destroy_code(&tag, &mut builder)
        .await?;
    
    Ok(())
}
```

### 6. 国际化 (i18n)

```rust
use lsys_core::fluents::{FluentMgr, FluentMessage};
use lsys_core::fluent_message;

// 初始化 Fluent
let fluent = FluentMgr::new("locale", "zh_CN").await?;

// 创建消息
let msg = fluent_message!("user-not-found", {
    "username": "admin"
});

// 格式化消息
let text = msg.default_format();
println!("{}", text); // 输出: 用户 admin 不存在
```

## 高级功能

以下功能适用于分布式系统和多节点部署场景。

### 分布式任务派发 (task_dispatch)

基于 Redis 的任务派发系统，将任务分发到多个主机上分别执行。

```rust
use lsys_core::task_dispatch::{TaskNotify, TaskNotifyConfig};
use std::sync::Arc;

// 配置任务
let task_config = Arc::new(TaskNotifyConfig::new("email_task"));

// 创建任务通知器
let task_notify = Arc::new(TaskNotify::new(
    redis_pool.clone(),
    task_config.clone(),
));

// 触发任务派发
task_notify.notify().await?;

// 定时触发任务
let timeout_notify = TaskTimeOutNotify::new(
    app_core.clone(),
    task_notify.clone(),
    time_out_notify.clone(),
    dispatch_config.clone(),
);

// 在指定时间触发
let run_time = now_time().unwrap() + 3600; // 1小时后
timeout_notify.notify_at_time(run_time).await?;
```

### 多节点消息同步 (remote_notify)

基于 Redis 的跨节点消息通知系统，将消息广播到所有节点。

```rust
use lsys_core::remote_notify::{RemoteNotify, RemoteTask, LocalExecType, ReplyWait};
use async_trait::async_trait;
use serde_json::Value;

// 定义远程任务
struct CacheClearTask;

#[async_trait]
impl RemoteTask for CacheClearTask {
    fn msg_type(&self) -> u8 {
        101 // 自定义消息类型
    }
    
    async fn run(&self, msg: MsgSendBody) -> Result<Option<Value>, String> {
        // 执行清理缓存逻辑
        println!("清理缓存: {:?}", msg.data);
        Ok(None)
    }
}

// 创建远程通知器
let remote_notify = Arc::new(RemoteNotify::new(
    "app_channel",
    app_core.clone(),
    redis_pool.clone(),
)?);

// 注册任务处理器
remote_notify.push_run(Box::new(CacheClearTask)).await;

// 启动监听（在后台任务中运行）
tokio::spawn({
    let notify = remote_notify.clone();
    let cancel_token = task_root.cancel_token();
    async move {
        notify.listen(cancel_token).await;
    }
});

// 发送消息到所有节点
remote_notify.call(
    101,                           // 消息类型
    serde_json::json!({"key": "user:123"}),
    None,                          // 目标主机（None = 所有节点）
    LocalExecType::RemoteExec,     // 本机也通过消息执行
    None,                          // 不等待返回
).await?;

// 发送消息并等待返回
let results = remote_notify.call(
    101,
    serde_json::json!({"action": "sync"}),
    None,
    LocalExecType::LocalExec,      // 本机直接执行
    Some(ReplyWait {
        max_node: 5,               // 最多等待5个节点
        timeout: 3000,             // 超时3秒
    }),
).await?;

for result in results {
    println!("节点 {} 返回: {:?}", result.from_host, result.data);
}
```

### 远程任务执行 (listen_notify)

基于 Redis 队列实现，任意一个主机派发任务到队列并监听结果，由监听队列的节点执行后将结果返回到派发任务的主机。

**使用场景：** 执行短信发送、邮件发送等任务，完成后将结果返回到申请发送的主机。

```rust
use lsys_core::listen_notify::{WaitItem, WaitNotify, WaitNotifyResult};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

// 1. 定义任务数据结构
#[derive(Debug, Serialize, Deserialize)]
struct SendTaskItem {
    task_id: u64,
    message_id: u64,
}

// 2. 实现 WaitItem trait
impl WaitItem for SendTaskItem {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id 
            || (self.message_id == other.message_id && other.task_id == 0)
    }
}

// 3. 创建通知器
let wait_notify = Arc::new(WaitNotify::<SendTaskItem>::new(
    "sms_task",            // 通道名称
    redis_pool.clone(),
    app_core.clone(),
    10,                    // 清理超时（秒）
));

// 4. 启动监听（在后台任务中）
tokio::spawn({
    let notify = wait_notify.clone();
    let cancel_token = task_root.cancel_token();
    async move {
        notify.listen(cancel_token).await;
    }
});

// 5. 派发任务并等待结果
async fn send_message(
    wait_notify: Arc<WaitNotify<SendTaskItem>>,
    task_id: u64,
) -> Result<WaitNotifyResult, Box<dyn std::error::Error>> {
    // 创建等待接收器
    let receiver = wait_notify.wait(SendTaskItem {
        task_id,
        message_id: 0,
    }).await;
    
    // 等待结果（带超时）
    let result = wait_notify.wait_timeout(receiver).await?;
    Ok(result)
}

// 6. 执行节点完成任务后通知结果
async fn notify_task_result(
    wait_notify: Arc<WaitNotify<SendTaskItem>>,
    host: &str,
    task_id: u64,
    result: WaitNotifyResult,
) -> Result<(), Box<dyn std::error::Error>> {
    wait_notify.notify(
        host,
        SendTaskItem { task_id, message_id: 0 },
        result,
    ).await?;
    Ok(())
}
```

### 分布式任务锁 (timeout_task)

基于 Redis 的分布式任务锁，确保多个节点下同一任务仅有一个节点执行。适用于定时任务、周期性任务等场景。

**使用场景：** 定时发送邮件、清理过期数据、同步任务等需要在多节点中只执行一次的任务。

```rust
use lsys_core::timeout_task::{
    TimeOutTask, TimeOutTaskConfig, TimeOutTaskNotify,
    TimeOutTaskExec, TimeOutTaskExecutor, TimeOutTaskNextTime
};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

// 1. 定义任务执行器
struct DataCleanTask {
    db: Pool<MySql>,
}

// 2. 实现 TimeOutTaskExec trait（执行逻辑）
#[async_trait::async_trait]
impl TimeOutTaskExec for DataCleanTask {
    async fn exec(
        &self,
        max_lock_time: usize,
        mut expire_call: impl FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,
    ) -> Result<(), String> {
        let start_time = now_time().unwrap_or_default();
        
        // 执行清理逻辑
        let result = sqlx::query("DELETE FROM logs WHERE created_at < ?")
            .bind(start_time - 86400 * 30) // 删除30天前的日志
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        
        // 如果执行时间超过锁时间的一半，延长锁
        let elapsed = now_time().unwrap_or_default() - start_time;
        if elapsed * 2 > max_lock_time as u64 {
            expire_call().await;
        }
        
        Ok(())
    }
}

// 3. 实现 TimeOutTaskNextTime trait（下次执行时间）
#[async_trait::async_trait]
impl TimeOutTaskNextTime for DataCleanTask {
    async fn next_time(&self, max_lock_time: usize) -> Result<Option<u64>, String> {
        let now = now_time().unwrap_or_default();
        
        // 查询下一个需要处理的时间
        let next_time = sqlx::query_scalar::<_, u64>(
            "SELECT MIN(expire_time) FROM tasks WHERE status = ? AND expire_time <= ?"
        )
        .bind(0)
        .bind(now + max_lock_time as u64)
        .fetch_one(&self.db)
        .await;
        
        match next_time {
            Ok(time) => Ok(Some(time)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}

// 4. 实现 TimeOutTaskExecutor trait
#[async_trait::async_trait]
impl TimeOutTaskExecutor for DataCleanTask {
    type Exec = Self;
    type NextTime = Self;
}

// 5. 创建并启动任务
async fn start_timeout_task(
    app_core: Arc<AppCore>,
    redis: deadpool_redis::Pool,
    db: Pool<MySql>,
) {
    // 创建任务配置
    let task_config = Arc::new(TimeOutTaskConfig::new(
        "data_clean_task",     // 任务名称
        60,                    // 最大锁定时间（秒）
    ));
    
    // 创建超时通知器
    let timeout_notify = Arc::new(TimeOutTaskNotify::new(
        redis,
        task_config,
    ));
    
    // 创建任务执行器
    let task_executor = Arc::new(DataCleanTask { db });
    
    // 启动任务监听（在后台任务中）
    tokio::spawn(async move {
        TimeOutTask::<DataCleanTask>::new(
            app_core,
            timeout_notify,
            task_executor.clone(),  // exec
            task_executor,          // next_time
        )
        .listen(None)  // channel_buffer
        .await;
    });
}
```
