import type gsap from "gsap";

/** 单实例 Tween 引用，便于在 effect 清理时 kill */
export class TweenHandle {
  private tween: gsap.core.Tween | null = null;

  set(next: gsap.core.Tween | null) {
    this.kill();
    this.tween = next;
  }

  kill() {
    this.tween?.kill();
    this.tween = null;
  }

  isActive() {
    return this.tween != null && this.tween.isActive();
  }
}

/** 多实例 Tween 槽位 */
export function createTweenHandles<const K extends string>(keys: readonly K[]) {
  return Object.fromEntries(keys.map((key) => [key, new TweenHandle()])) as Record<K, TweenHandle>;
}
