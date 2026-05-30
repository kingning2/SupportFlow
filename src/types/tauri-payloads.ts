import type { FeLogLevel } from "@/enums";

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
