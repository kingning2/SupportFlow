"use client";

import { useRouter } from "next/navigation";

import AppErrorView from "@/components/error/app-error-view";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export default function Error({ error, reset }: ErrorProps) {
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
