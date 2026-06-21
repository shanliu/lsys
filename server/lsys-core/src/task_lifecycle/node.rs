use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use parking_lot::Mutex;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use super::report::{NodeReport, TaskOutcome};

/// 任务节点（树状结构）
///
/// 每个节点可以：
/// - 持有多个本地任务（通过 [`spawn`](Self::spawn) 注册）
/// - 拥有子节点（通过 [`child`](Self::child) 创建）
/// - 被取消时，先依次关闭子节点，再关闭本地任务
///
/// 取消令牌通过 `CancellationToken` 的父子关系自动传播：
/// 父节点 cancel → 所有子节点的 token 同时被取消。
/// 但 [`shutdown`](Self::shutdown) 的等待顺序是：先等子节点完成，再等本节点任务完成。
///
/// 信号监听、shutdown 顺序编排等应用层逻辑不在 core 职责范围内，
/// 由调用方自行组合（见 `examples/lsys-actix-web`）。
pub struct TaskNode {
    name: String,
    cancel_token: CancellationToken,
    children: Mutex<Vec<Arc<TaskNode>>>,
    tasks: Mutex<JoinSet<()>>,
    grace_period: Duration,
    shutting_down: AtomicBool,
}

impl TaskNode {
    /// 创建根节点
    pub fn root(name: impl Into<String>, grace_period: Duration) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            cancel_token: CancellationToken::new(),
            children: Mutex::new(Vec::new()),
            tasks: Mutex::new(JoinSet::new()),
            grace_period,
            shutting_down: AtomicBool::new(false),
        })
    }

    /// 创建子节点
    ///
    /// 子节点的 cancel_token 是父节点的 child_token，
    /// 父节点被取消时子节点自动被取消。
    pub fn child(self: &Arc<Self>, name: impl Into<String>) -> Arc<TaskNode> {
        let child = Arc::new(TaskNode {
            name: name.into(),
            cancel_token: self.cancel_token.child_token(),
            children: Mutex::new(Vec::new()),
            tasks: Mutex::new(JoinSet::new()),
            grace_period: self.grace_period,
            shutting_down: AtomicBool::new(false),
        });
        self.children.lock().push(child.clone());
        child
    }

    /// 获取本节点的取消令牌（用于任务内部 select cancel）
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// 本节点是否已被取消
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// 启动任务
    ///
    /// 闭包接收 `CancellationToken`，任务内部应通过 select 监听取消。
    /// 同一节点可以多次 spawn，所有任务都会被追踪。
    ///
    /// 关闭期间调用 spawn 会被忽略（不会创建孤儿任务）。
    pub fn spawn<F, Fut>(self: &Arc<Self>, f: F)
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.shutting_down.load(Ordering::SeqCst) {
            warn!(
                target: "task_lifecycle",
                "spawn ignored on node '{}': already shutting down",
                self.name
            );
            return;
        }

        let token = self.cancel_token.child_token();
        self.tasks.lock().spawn(async move {
            f(token).await;
        });
    }

    /// 树状关闭（核心方法）
    ///
    /// 关闭顺序：
    /// 1. 标记 shutting_down，阻止新 spawn
    /// 2. 并行关闭所有子节点（每个子节点递归执行 shutdown）
    /// 3. 取消本节点的 token（通知本节点所有任务）
    /// 4. 等待本节点本地任务完成（超时则 abort）
    ///
    /// 注意：tokio 的 abort 是协作式的，如果任务卡在同步代码中
    /// （如阻塞 I/O），abort 无法立即生效。确保任务内部使用 async I/O。
    pub async fn shutdown(self: &Arc<Self>) -> NodeReport {
        let start = Instant::now();

        self.shutting_down.store(true, Ordering::SeqCst);

        info!(
            target: "task_lifecycle",
            "shutting down node '{}'",
            self.name
        );

        // 并行关闭子节点
        let children = self.children.lock().clone();
        let child_reports: Vec<NodeReport> =
            join_all(children.iter().map(|c| c.shutdown())).await;

        // 取消本节点 token
        self.cancel_token.cancel();

        // 等待本地任务完成
        let outcome = self.drain_tasks().await;

        let report = NodeReport {
            name: self.name.clone(),
            outcome,
            duration: start.elapsed(),
            children: child_reports,
        };

        let (completed, timed_out, panicked) = report.count_summary();
        info!(
            target: "task_lifecycle",
            "node '{}' shutdown complete: {} completed, {} timed_out, {} panicked, {:?} total",
            self.name, completed, timed_out, panicked, report.duration
        );

        report
    }

    /// 等待本地 JoinSet 中的所有任务完成
    ///
    /// - 在 grace_period 内正常完成 → Completed
    /// - 超时后 abort 所有剩余任务 → TimedOut
    /// - panic 的任务 → Panicked
    async fn drain_tasks(&self) -> Option<TaskOutcome> {
        let mut joinset: JoinSet<()> = {
            let mut tasks = self.tasks.lock();
            std::mem::take(&mut *tasks)
        };

        if joinset.is_empty() {
            return None;
        }

        let deadline = tokio::time::Instant::now() + self.grace_period;
        let mut any_timed_out = false;
        let mut any_panicked = false;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());

            if remaining.is_zero() {
                joinset.abort_all();
                any_timed_out = true;
                // abort 后给 1s 硬超时收尾，防止卡在同步代码中的任务无限阻塞
                let _ = tokio::time::timeout(Duration::from_secs(1), async {
                    while joinset.join_next().await.is_some() {}
                }).await;
                break;
            }

            match tokio::time::timeout(remaining, joinset.join_next()).await {
                Ok(Some(Ok(()))) => {}
                Ok(Some(Err(e))) => {
                    if e.is_panic() {
                        any_panicked = true;
                        warn!(
                            target: "task_lifecycle",
                            "task on node '{}' panicked: {:?}",
                            self.name, e
                        );
                    }
                    if e.is_cancelled() {
                        any_timed_out = true;
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    joinset.abort_all();
                    any_timed_out = true;
                    let _ = tokio::time::timeout(Duration::from_secs(1), async {
                        while joinset.join_next().await.is_some() {}
                    }).await;
                    break;
                }
            }
        }

        Some(if any_panicked {
            TaskOutcome::Panicked("see logs above".into())
        } else if any_timed_out {
            TaskOutcome::TimedOut
        } else {
            TaskOutcome::Completed
        })
    }
}
