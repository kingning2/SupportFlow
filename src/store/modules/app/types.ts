import type { Language } from "@/enums";

export interface AppInitialState {
  initialized: boolean;
  titleBarHeight: number;
  mainWindowGlobalGg: string;
  supportLanguages: { label: string; value: Language }[];
  currentLanguage: Language;
}
