import { Language, TauriCmd } from "../enums";

import { invokeWrapper } from "./invoke";

export async function getLanguageResourceBundle(currentLanguage: Language) {
  return invokeWrapper<Record<string, Record<string, unknown>>>(
    TauriCmd.GetLanguageResourceBundle,
    {
      language: currentLanguage
    }
  );
}

export async function getLang(): Promise<Language> {
  return invokeWrapper<Language>(TauriCmd.GetLang);
}

export async function setLang(lang: Language): Promise<void> {
  await invokeWrapper<void>(TauriCmd.SetLang, { lang });
}
