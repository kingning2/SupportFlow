/** Frontend log levels (mirrors `@supportflow/shared/tauri-bridge` / Rust). */
export type FeLogLevel = "error" | "warn" | "info" | "debug";

export type ModalLifecyclePayload = {
  label: string;
};

export type ModalOpenPanelPayload = {
  name: string;
  title?: string;
};

export type FeLogPayload = {
  level: FeLogLevel;
  msg: string;
};

/** Python sidecar → Tauri (`channel/status-changed`). */
export type ChannelStatusChangedPayload = {
  channel: string;
  phase: string;
  message?: string;
  userId?: string;
  displayName?: string;
  waitSeconds?: number;
};
