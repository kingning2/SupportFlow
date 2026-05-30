"use client";

import { useTranslation } from "react-i18next";

import { useModalWindow } from "@/components/modal";
import { Button } from "@/components/ui/button";
import { ModalPanel } from "@/enums";

export default function MainWindowHome() {
  const { t } = useTranslation("home");
  const { openModal } = useModalWindow();

  return (
    <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-4 p-6">
      <h1 className="text-foreground text-2xl font-semibold tracking-tight">{t("title")}</h1>
      <p className="text-muted-foreground max-w-md text-center text-sm leading-relaxed">
        {t("description")}
      </p>
      <Button
        type="button"
        onClick={() =>
          void openModal({
            name: ModalPanel.Demo,
            title: t("open_demo_modal"),
            width: 480,
            height: 360
          })
        }
      >
        {t("open_demo_modal")}
      </Button>
    </div>
  );
}
