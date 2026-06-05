"use client";

import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { log } from "@supportflow/shared/tauri-bridge/cmd/log";
import { FeLogLevel } from "@supportflow/shared/tauri-bridge/enums";
import { Button } from "@supportflow/ui/button";
import { Card, CardContent } from "@supportflow/ui/card";

type AppErrorViewProps = {
  error: Error & { digest?: string };
  reset: () => void;
  logPrefix: string;
  onBack?: () => void;
  backLabelKey?: "error_back_home" | "error_close";
};

export default function AppErrorView({
  error,
  reset,
  logPrefix,
  onBack,
  backLabelKey = "error_back_home"
}: AppErrorViewProps) {
  const { t } = useTranslation("common");

  useEffect(() => {
    void log(FeLogLevel.Error, `${logPrefix}: ${error.message || String(error)}`);
  }, [error, logPrefix]);

  const message = error.message || String(error);

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardContent className="pt-6">
          <div className="text-center">
            <div className="text-lg font-semibold">{t("error_title")}</div>
            <div className="text-muted-foreground mt-2 text-sm wrap-break-word">{message}</div>
            {error.digest ? (
              <div className="text-muted-foreground/80 mt-2 text-xs wrap-break-word">
                {t("error_digest", { digest: error.digest })}
              </div>
            ) : null}
          </div>

          <div className="mt-6 flex flex-col gap-2 sm:flex-row sm:justify-center">
            {onBack ? (
              <Button variant="secondary" onClick={onBack}>
                {t(backLabelKey)}
              </Button>
            ) : null}
            <Button onClick={reset}>{t("error_retry")}</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
