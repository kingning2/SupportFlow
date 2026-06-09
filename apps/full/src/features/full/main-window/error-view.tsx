"use client";

import { useRouter } from "next/navigation";

import AppErrorView from "@/features/full/shared/app-error-view";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export function ErrorView({ error, reset }: ErrorProps) {
  const router = useRouter();

  return (
    <AppErrorView
      error={error}
      reset={reset}
      logPrefix="main-window err capture"
      onBack={() => router.replace("/main-window")}
    />
  );
}
