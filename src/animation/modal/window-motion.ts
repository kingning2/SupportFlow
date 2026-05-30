/** Modal 子窗口进场/退场（scale + opacity，避免在 #App clip-path 内被裁切） */
export const MODAL_ENTER_FROM = {
  scale: 0.96,
  opacity: 0,
  duration: 0.34,
  ease: "power2.out"
} as const;

export const MODAL_EXIT_TO = {
  scale: 0.96,
  opacity: 0,
  duration: 0.26,
  ease: "power2.in"
} as const;

export function prefersReducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}
