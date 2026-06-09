"use client";

import { ErrorView } from "@/features/full/main-window/error-view";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export default function Error({ error, reset }: ErrorProps) {
  return <ErrorView error={error} reset={reset} />;
}
