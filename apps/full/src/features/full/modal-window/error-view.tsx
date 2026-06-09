"use client";

import { useModalMotion } from "@supportflow/ui/modal";

import AppErrorView from "@/features/full/shared/app-error-view";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export function ErrorView({ error, reset }: ErrorProps) {
  const { requestClose } = useModalMotion();

  return (
    <AppErrorView
      error={error}
      reset={reset}
      logPrefix="modal-window err capture"
      onBack={() => {
        requestClose();
      }}
      backLabelKey="error_close"
    />
  );
}
