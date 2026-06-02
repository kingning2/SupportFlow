import { Language } from "@supportflow/shared/tauri-bridge/enums";

export type AppConfig = {
  titleBarHeight: number;
  mainWindowGlobalGg: string;
  defaultLanguage: Language;
  supportLanguages: Array<{ label: string; value: Language }>;
};

export const appConfig: AppConfig = {
  titleBarHeight: 40,
  mainWindowGlobalGg: "#f0f4f8",
  defaultLanguage: Language.Cn,
  supportLanguages: [
    { label: "繁體中文", value: Language.Cn },
    { label: "English", value: Language.En }
  ]
};
