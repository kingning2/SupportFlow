import type { Language } from "@supportflow/shared/tauri-bridge/enums";

export interface AppInitialState {
  initialized: boolean;
  titleBarHeight: number;
  mainWindowGlobalGg: string;
  supportLanguages: { label: string; value: Language }[];
  currentLanguage: Language;
}
