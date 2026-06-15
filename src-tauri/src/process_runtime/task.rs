//! 当前 async 任务范围内的进程上下文。

use tokio::task_local;

task_local! {
    static CURRENT_TASK: ProcessTaskContext;
}

/// 绑定到单个 IO / RPC 任务上的进程标识。
#[derive(Clone, Copy, Debug)]
pub struct ProcessTaskContext {
    pub process_name: &'static str,
}

impl ProcessTaskContext {
    pub async fn scope<F, T>(ctx: Self, fut: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        CURRENT_TASK.scope(ctx, fut).await
    }

    pub fn current_process_name() -> Option<&'static str> {
        CURRENT_TASK.try_with(|ctx| ctx.process_name).ok()
    }
}
