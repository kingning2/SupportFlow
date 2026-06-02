import { MODAL_ENTER_FROM, MODAL_EXIT_TO, prefersReducedMotion } from "./window-motion";

export function getModalMotionTarget(): HTMLElement | null {
  if (typeof document === "undefined") return null;
  return document.getElementById("App");
}

function waitFrames(count: number): Promise<void> {
  return new Promise((resolve) => {
    let left = count;
    const step = () => {
      left -= 1;
      if (left <= 0) resolve();
      else requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
  });
}

/** show 之后播放进场；优先 WAAPI（子 Webview 内 GSAP 偶发不写 transform） */
export async function playModalEnter(target: HTMLElement): Promise<void> {
  if (prefersReducedMotion()) {
    target.style.opacity = "1";
    target.style.transform = "none";
    return;
  }

  await waitFrames(2);

  target.getAnimations().forEach((a) => a.cancel());
  target.style.transformOrigin = "50% 50%";

  const animation = target.animate(
    [
      { opacity: MODAL_ENTER_FROM.opacity, transform: `scale(${MODAL_ENTER_FROM.scale})` },
      { opacity: 1, transform: "scale(1)" }
    ],
    {
      duration: MODAL_ENTER_FROM.duration * 1000,
      easing: "cubic-bezier(0.22, 1, 0.36, 1)",
      fill: "forwards"
    }
  );

  try {
    await animation.finished;
  } catch {
    /* cancelled */
  }
}

export function playModalExit(target: HTMLElement, onComplete: () => void): void {
  if (prefersReducedMotion()) {
    onComplete();
    return;
  }

  target.getAnimations().forEach((a) => a.cancel());
  target.style.transformOrigin = "50% 50%";

  const animation = target.animate(
    [
      { opacity: 1, transform: "scale(1)" },
      { opacity: MODAL_EXIT_TO.opacity, transform: `scale(${MODAL_EXIT_TO.scale})` }
    ],
    {
      duration: MODAL_EXIT_TO.duration * 1000,
      easing: "cubic-bezier(0.4, 0, 1, 1)",
      fill: "forwards"
    }
  );

  animation.onfinish = () => onComplete();
  animation.oncancel = () => onComplete();
}
