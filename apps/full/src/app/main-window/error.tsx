"use client";

import { FullMainWindowErrorView } from "@/features/full/main-window/main-window-error-view";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export default function Error({ error, reset }: ErrorProps) {
  return <FullMainWindowErrorView error={error} reset={reset} />;
}
