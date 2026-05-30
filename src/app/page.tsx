"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

import { useAppSelector } from "@/store/hooks";

export default function Root() {
  const router = useRouter();
  const initialized = useAppSelector((state) => state.app.initialized);

  useEffect(() => {
    if (!initialized) return;
    router.replace("/main-window");
  }, [initialized, router]);

  return null;
}
