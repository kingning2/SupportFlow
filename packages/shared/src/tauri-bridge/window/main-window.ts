import { LogicalSize, Window } from "@tauri-apps/api/window";

import { WindowLabel } from "../enums/window-label";

/** 固定 label，供标题栏拖动与窗口 API 使用 */
export const mainWindow = new Window(WindowLabel.Main);

export type MainWindowSize = {
  width: number;
  height: number;
};

const DEFAULT_SIZE: MainWindowSize = { width: 1200, height: 800 };

/**
 * 在 WebView 内注册主窗体行为：
 * - DPI / 缩放变化时恢复逻辑尺寸
 * - 关闭请求时销毁所有窗口
 */
export function initWindowConfig(size: MainWindowSize = DEFAULT_SIZE) {
  if (typeof window === "undefined") return;

  void mainWindow.onScaleChanged(() => {
    void mainWindow.setSize(new LogicalSize(size.width, size.height));
  });

  void mainWindow.onCloseRequested(async () => {
    const allWindow = await Window.getAll();
    await Promise.all(allWindow.map((w) => w.destroy()));
  });
}
