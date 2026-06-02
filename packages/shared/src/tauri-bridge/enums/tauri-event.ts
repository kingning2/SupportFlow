/** Tauri 事件名（与 Rust `events::names`、前端监听处保持一致） */
export enum TauriEvent {
  SessionChanged = "session/changed",
  ModalOpened = "modal/opened",
  ModalClosed = "modal/closed",
  ModalOpenPanel = "modal/open-panel",
  FeLog = "fe/log",
  FeLogReq = "fe/log-req",
  AgentStreamChunk = "agent/stream-chunk",
  AgentRunFinished = "agent/run-finished",
  AgentLogStream = "agent/log-stream"
}
