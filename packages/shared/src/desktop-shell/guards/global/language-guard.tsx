"use client";

import { useEffect, useState } from "react";
import i18next from "i18next";

import { getLanguageResourceBundle } from "@supportflow/shared/tauri-bridge/cmd/lang";
import "../../config/i18n";
import { useAppSelector } from "../../store/hooks";

export default function LanguageGuard({ children }: { children: React.ReactNode }) {
  const currentLanguage = useAppSelector((state) => state.app.currentLanguage);
  const [readyLanguage, setReadyLanguage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    getLanguageResourceBundle(currentLanguage)
      .catch((err) => {
        console.error("get_language_resource_bundle", err);
        return {};
      })
      .then((resource: Record<string, Record<string, unknown>>) => {
        if (cancelled) return;
        Object.keys(resource).forEach((key) => {
          i18next.addResourceBundle(currentLanguage, key, resource[key], true, true);
        });
        void i18next.changeLanguage(currentLanguage);
        setReadyLanguage(currentLanguage);
      });

    return () => {
      cancelled = true;
    };
  }, [currentLanguage]);

  return readyLanguage === currentLanguage ? <>{children}</> : null;
}
