"use client";

import { FullModalWindowErrorView } from "@/features/full/modal-window/modal-window-error-view";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export default function Error({ error, reset }: ErrorProps) {
  return <FullModalWindowErrorView error={error} reset={reset} />;
}
