"use client";

import { memo } from "react";

import { useIsomorphicLayoutEffect } from "@supportflow/shared/desktop-shell/useIsomorphicLayoutEffect";
import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

/** 将 Redux 中的主窗背景同步到 `:root` 的 `--main-window-bg` */
export const MainWindowBgGuard = memo(function MainWindowBgGuard() {
  const globalGg = useAppSelector((state) => state.app.mainWindowGlobalGg);

  useIsomorphicLayoutEffect(() => {
    document.documentElement.style.setProperty("--main-window-bg", globalGg);
  }, [globalGg]);

  return null;
});
