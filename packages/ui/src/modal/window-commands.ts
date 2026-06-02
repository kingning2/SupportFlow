import {
  closeModalWindow,
  openModalWindow,
  preloadModalWindow
} from "@supportflow/shared/tauri-bridge/cmd/window";
import { AppRoute, WindowLabel, type ModalPanel } from "@supportflow/shared/tauri-bridge/enums";

export { TauriEvent, WindowLabel } from "@supportflow/shared/tauri-bridge/enums";

export type OpenModalWindowOptions = {
  /** 面板组件名，对应应用内 modal panels 注册表 */
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
