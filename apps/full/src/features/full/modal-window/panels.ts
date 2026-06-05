"use client";

import type { ComponentType } from "react";

import { ModalPanel, isModalPanel } from "@supportflow/shared/tauri-bridge/enums";

import { DemoPanel } from "./panels/demo-panel";

export const MODAL_PANEL_REGISTRY: Record<ModalPanel, ComponentType> = {
  [ModalPanel.Demo]: DemoPanel
};

export { ModalPanel };

export function resolveModalPanel(name: string): ComponentType | undefined {
  if (!isModalPanel(name)) return undefined;
  return MODAL_PANEL_REGISTRY[name];
}
