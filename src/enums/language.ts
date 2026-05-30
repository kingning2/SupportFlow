export enum Language {
  Cn = "cn",
  En = "en"
}

export function isLanguage(value: string): value is Language {
  return value === Language.Cn || value === Language.En;
}
