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
 * 同步主窗口最大化状态到根节点 class，便于桌面壳切换圆角。
 *
 * # Arguments
 *
 * * `root` - 主窗口根节点
 */
async function syncMaximizedClass(root: HTMLElement) {
  const maximized = await mainWindow.isMaximized();
  root.classList.toggle("window-maximized", maximized);
}

/**
 * 在 WebView 内注册主窗体行为：
 * - DPI / 缩放变化时恢复逻辑尺寸
 * - 最大化切换时同步根节点样式
 * - 关闭请求时销毁所有窗口
 *
 * # Arguments
 *
 * * `size` - 主窗口默认逻辑尺寸
 */
export function initWindowConfig(size: MainWindowSize = DEFAULT_SIZE) {
  if (typeof window === "undefined") return;

  const root = document.getElementById("App");

  void mainWindow.onScaleChanged(() => {
    void mainWindow.setSize(new LogicalSize(size.width, size.height));
  });

  if (root) {
    void syncMaximizedClass(root);
    void mainWindow.onResized(() => {
      void syncMaximizedClass(root);
    });
  }

  void mainWindow.onCloseRequested(async () => {
    const allWindow = await Window.getAll();
    await Promise.all(allWindow.map((w) => w.destroy()));
  });
}
