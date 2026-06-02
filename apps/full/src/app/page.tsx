"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

import { AppRoute } from "@supportflow/shared/tauri-bridge/enums/app-route";
import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

/** 完整控制台入口（渠道独立应用见 apps/wework、apps/wechat） */
export default function Root() {
  const router = useRouter();
  const initialized = useAppSelector((state) => state.app.initialized);

  useEffect(() => {
    if (!initialized) return;
    router.replace(AppRoute.MainWindow);
  }, [initialized, router]);

  return null;
}
