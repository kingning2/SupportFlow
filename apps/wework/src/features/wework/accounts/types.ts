/** 企微通道 connect 使用的配置字段 */
export type WeworkAccountConfig = {
  wework_exe_path?: string;
  wework_version?: string;
  wework_smart?: boolean;
  wework_init_wait_seconds?: number;
};

export interface WeworkSavedAccount {
  id: string;
  /** 列表展示名（企微登录 nickname / username） */
  label: string;
  config: WeworkAccountConfig;
  createdAt: number;
  lastConnectedAt?: number;
  /** 企微 user_id，用于同账号去重 */
  weworkUserId?: string;
}
