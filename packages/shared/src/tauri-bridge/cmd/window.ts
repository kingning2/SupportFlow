import { TauriCmd } from "../enums";

import { invokeWrapper } from "./invoke";

export type OpenModalWindowArgs = {
  path: string;
  title?: string;
  width?: number;
  height?: number;
  label?: string;
};

export const openModalWindow = (args: OpenModalWindowArgs) =>
  invokeWrapper<string>(TauriCmd.OpenModalWindow, { ...args });

export const closeModalWindow = (label: string) =>
  invokeWrapper<void>(TauriCmd.CloseModalWindow, { label });

export const notifyModalWindowReady = (label: string) =>
  invokeWrapper<void>(TauriCmd.ModalWindowReady, { label });

/** 主窗空闲时后台预热隐藏 modal Webview（不显示、不触发蒙层） */
export const preloadModalWindow = () => invokeWrapper<void>(TauriCmd.PreloadModalWindow);
