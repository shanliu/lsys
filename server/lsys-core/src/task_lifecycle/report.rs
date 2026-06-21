use std::fmt;
use std::time::Duration;

/// 单个任务节点的关闭结果
#[derive(Debug, Clone)]
pub enum TaskOutcome {
    /// 任务正常完成
    Completed,
    /// 任务在 grace_period 内未退出，被强制 abort
    TimedOut,
    /// 任务发生 panic
    Panicked(String),
}

impl fmt::Display for TaskOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskOutcome::Completed => write!(f, "completed"),
            TaskOutcome::TimedOut => write!(f, "timed_out"),
            TaskOutcome::Panicked(msg) => write!(f, "panicked: {msg}"),
        }
    }
}

/// 单个节点的关闭报告
#[derive(Debug, Clone)]
pub struct NodeReport {
    /// 节点名称
    pub name: String,
    /// 本节点本地任务的关闭结果（None = 分支节点无本地任务）
    pub outcome: Option<TaskOutcome>,
    /// 本节点关闭耗时
    pub duration: Duration,
    /// 子节点报告
    pub children: Vec<NodeReport>,
}

impl NodeReport {
    /// 统计所有节点的完成情况
    pub fn count_summary(&self) -> (usize, usize, usize) {
        let mut completed = 0;
        let mut timed_out = 0;
        let mut panicked = 0;

        match &self.outcome {
            None => {}
            Some(TaskOutcome::Completed) => completed += 1,
            Some(TaskOutcome::TimedOut) => timed_out += 1,
            Some(TaskOutcome::Panicked(_)) => panicked += 1,
        }

        for child in &self.children {
            let (c, t, p) = child.count_summary();
            completed += c;
            timed_out += t;
            panicked += p;
        }

        (completed, timed_out, panicked)
    }

    /// 树状打印关闭报告（使用 tracing::info）
    pub fn log_tree(&self, indent: usize) {
        let prefix = "  ".repeat(indent);
        let status = match &self.outcome {
            None => "[branch]".to_string(),
            Some(TaskOutcome::Completed) => "[ok]".to_string(),
            Some(TaskOutcome::TimedOut) => "[timeout]".to_string(),
            Some(TaskOutcome::Panicked(msg)) => format!("[panic: {}]", msg),
        };

        tracing::info!(
            target: "task_lifecycle",
            "{}{} {} - {:?}",
            prefix, status, self.name, self.duration
        );

        for child in &self.children {
            child.log_tree(indent + 1);
        }
    }
}
