import { LocalCacheKey } from "@/enums";

export type CowTheme = "light" | "dark";

export function readCowTheme(): CowTheme {
  if (typeof window === "undefined") {
    return "dark";
  }
  return localStorage.getItem(LocalCacheKey.CowTheme) === "light" ? "light" : "dark";
}

export function applyCowTheme(theme: CowTheme): void {
  localStorage.setItem(LocalCacheKey.CowTheme, theme);
  document.documentElement.classList.toggle("dark", theme === "dark");
}

export function toggleCowTheme(): CowTheme {
  const next: CowTheme = readCowTheme() === "dark" ? "light" : "dark";
  applyCowTheme(next);
  return next;
}
