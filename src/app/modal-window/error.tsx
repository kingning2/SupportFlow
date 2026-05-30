"use client";

import AppErrorView from "@/components/error/app-error-view";
import { useModalMotion } from "@/components/modal";

type ErrorProps = {
  error: Error & { digest?: string };
  reset: () => void;
};

export default function Error({ error, reset }: ErrorProps) {
  const { requestClose } = useModalMotion();

  const handleBack = () => {
    requestClose();
  };

  return (
    <AppErrorView
      error={error}
      reset={reset}
      logPrefix="modal-window err capture"
      onBack={handleBack}
      backLabelKey="error_close"
    />
  );
}
