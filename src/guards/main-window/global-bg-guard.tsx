"use client";

import { memo } from "react";

import { useIsomorphicLayoutEffect } from "@/hooks/useIsomorphicLayoutEffect";
import { useAppSelector } from "@/store/hooks";

/** 将 Redux 中的主窗背景同步到 `:root` 的 `--main-window-bg`（与 unlock 的 GlobalBgGuard 一致） */
const GlobalBgGuard = memo(() => {
  const globalGg = useAppSelector((state) => state.app.mainWindowGlobalGg);

  useIsomorphicLayoutEffect(() => {
    document.documentElement.style.setProperty("--main-window-bg", globalGg);
  }, [globalGg]);

  return null;
});

export default GlobalBgGuard;
