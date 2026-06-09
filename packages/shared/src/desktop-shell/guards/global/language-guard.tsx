"use client";

import { useEffect, useState } from "react";
import i18next from "i18next";

import { useAppSelector } from "../../store/hooks";

export default function LanguageGuard({ children }: { children: React.ReactNode }) {
  const currentLanguage = useAppSelector((state) => state.app.currentLanguage);
  const [readyLanguage, setReadyLanguage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    return () => {
      cancelled = true;
    };
  }, [currentLanguage]);

  return readyLanguage === currentLanguage ? <>{children}</> : null;
}
