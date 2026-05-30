"use client";

import { Bot } from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

export function ConsoleBrandMark({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "flex shrink-0 items-center justify-center rounded-xl bg-[#35A85B] shadow-lg shadow-[#35A85B]/20",
        className
      )}
    >
      <Bot className="size-[55%] text-white" />
    </div>
  );
}

export function ConsoleBrandMarkSmall({ className }: { className?: string }) {
  return (
    <div
      className={cn("flex shrink-0 items-center justify-center rounded-lg bg-[#35A85B]", className)}
    >
      <Bot className="size-[55%] text-white" />
    </div>
  );
}

export function ViewShell({
  title,
  description,
  children
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="border-b border-slate-200 p-3 dark:border-white/10">
        <h2 className="text-lg font-semibold text-slate-800 dark:text-slate-100">{title}</h2>
        {description ? (
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{description}</p>
        ) : null}
      </div>
      <div className="min-h-0 flex-1 overflow-auto p-6">{children}</div>
    </div>
  );
}

export function PlaceholderView({ viewKey }: { viewKey: string }) {
  const { t } = useTranslation("console");

  return (
    <ViewShell title={t(viewKey)} description={t("placeholder_body")}>
      <div className="text-muted-foreground text-sm">{t("placeholder_body")}</div>
    </ViewShell>
  );
}
