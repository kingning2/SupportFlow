import { LocalCacheKey } from "@/enums";

export type ConsoleTheme = "light" | "dark";

export function readConsoleTheme(): ConsoleTheme {
  if (typeof window === "undefined") {
    return "dark";
  }
  return localStorage.getItem(LocalCacheKey.ConsoleTheme) === "light" ? "light" : "dark";
}

export function applyConsoleTheme(theme: ConsoleTheme): void {
  localStorage.setItem(LocalCacheKey.ConsoleTheme, theme);
  document.documentElement.classList.toggle("dark", theme === "dark");
}

export function toggleConsoleTheme(): ConsoleTheme {
  const next: ConsoleTheme = readConsoleTheme() === "dark" ? "light" : "dark";
  applyConsoleTheme(next);
  return next;
}
