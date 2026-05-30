"use client";

import { useTranslation } from "react-i18next";

import { Modal } from "@/components/modal/modal";
import { useAppSelector } from "@/store/hooks";

export function DemoPanel() {
  const { t } = useTranslation("modal_window");
  const lang = useAppSelector((state) => state.app.currentLanguage);

  return (
    <Modal title={t("demo_title")} bodyClassName="flex flex-col gap-3">
      <p className="text-muted-foreground text-sm leading-relaxed">{t("demo_body")}</p>
      <p className="text-sm">{t("shared_session", { lang })}</p>
      <p className="text-muted-foreground text-xs">{t("hint")}</p>
    </Modal>
  );
}
