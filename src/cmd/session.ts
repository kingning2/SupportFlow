import { Language, TauriCmd } from "@/enums";
import type { AppSession as GeneratedAppSession } from "@/generated/contracts";

import { invokeWrapper } from ".";

/** 与 Rust `AppSession` 对齐；语言字段收窄为应用支持的语言。 */
export type AppSession = Omit<GeneratedAppSession, "currentLanguage"> & {
  currentLanguage: Language;
};

export const getAppSession = () => invokeWrapper<AppSession>(TauriCmd.GetAppSession);
