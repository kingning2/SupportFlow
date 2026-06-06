//! tokio 子进程后端句柄。

use tokio::process::{Child, ChildStdin};

pub enum ProcessBackend {
    Tokio {
        child: Child,
        stdin: ChildStdin,
    },
}
