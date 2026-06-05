//! Python sidecar 互操作：路径、MarkItDown、stdio RPC。

mod sidecar;

pub use sidecar::{spawn_sidecar, ChannelPythonSidecar};
