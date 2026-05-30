import { LogicalSize, Window } from "@tauri-apps/api/window";

import { closeModalWindow, openModalWindow, preloadModalWindow } from "@/cmd/window";
import type { ModalPanel } from "@/enums";
import { AppRoute, TauriEvent, WindowLabel } from "@/enums";

export { TauriEvent, WindowLabel };

/** 与 unlock-next-app 一致：固定 label，供标题栏拖动与窗口 API 使用 */
export const mainWindow = new Window(WindowLabel.Main);

/**
 * 在 WebView 内注册主窗体行为（与 unlock `global-provider` 里 `initWindowConfig()` 对齐）。
 * - DPI / 缩放变化时恢复逻辑尺寸，避免无边框窗在高分屏下尺寸漂移
 * - 关闭请求时销毁所有窗口（多窗场景下一并退出）
 */
export function initWindowConfig() {
  if (typeof window === "undefined") return;

  void mainWindow.onScaleChanged(() => {
    const logicalSize = new LogicalSize(1200, 800);
    void mainWindow.setSize(logicalSize);
  });

  void mainWindow.onCloseRequested(async () => {
    const allWindow = await Window.getAll();
    await Promise.all(allWindow.map((w) => w.destroy()));
  });
}

export type OpenModalWindowOptions = {
  /** 面板组件名，对应 `src/components/modal/panels` 注册表 */
  name: ModalPanel;
  title?: string;
  width?: number;
  height?: number;
  /** 固定为 `modal`；传入其它值也会被归一为单窗 label */
  label?: string;
};

function modalWindowPath(name: ModalPanel): string {
  return `${AppRoute.ModalWindow}?name=${encodeURIComponent(name)}`;
}

export function isModalWindowLabel(label: string): boolean {
  return label === WindowLabel.Modal || label.startsWith(`${WindowLabel.Modal}-`);
}

/** 通过 Rust command 打开 modal 子窗口（非前端 WebviewWindow API） */
export async function openModalWindowCommand(options: OpenModalWindowOptions): Promise<string> {
  return openModalWindow({
    path: modalWindowPath(options.name),
    title: options.title,
    width: options.width,
    height: options.height,
    label: options.label ?? WindowLabel.Modal
  });
}

export async function closeModalWindowCommand(label: string): Promise<void> {
  await closeModalWindow(label);
}

/** 主窗渲染稳定后在空闲时调用，避免首次打开 modal 卡顿 */
export async function preloadModalWindowCommand(): Promise<void> {
  await preloadModalWindow();
}
