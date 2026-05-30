"use client";

import { BookOpen, Clock, Code, FolderOpen, Globe, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";

import { EXAMPLE_PROMPTS } from "@/components/agent-console/constants/example-prompts";
import { ConsoleBrandMark } from "@/components/agent-console/shared/console-brand";
import { cn } from "@/lib/utils";

const EXAMPLE_ICONS = {
  sys: FolderOpen,
  task: Clock,
  code: Code,
  knowledge: BookOpen,
  skill: Zap,
  web: Globe
} as const;

interface WelcomeScreenProps {
  onSelectPrompt: (text: string) => void;
}

export function WelcomeScreen({ onSelectPrompt }: WelcomeScreenProps) {
  const { t } = useTranslation("console");

  return (
    <div
      className="flex h-full flex-col items-center justify-center px-6 pb-16"
      style={{ paddingTop: "6vh" }}
    >
      <ConsoleBrandMark className="mb-6 size-16" />
      <h1 className="mb-3 text-2xl font-bold text-slate-800 dark:text-slate-100">
        {t("welcome_title")}
      </h1>
      <p className="mb-10 max-w-lg text-center leading-relaxed text-slate-500 dark:text-slate-400">
        {t("welcome_subtitle")}
      </p>

      <div className="grid w-full max-w-2xl grid-cols-2 gap-3 sm:grid-cols-3">
        {EXAMPLE_PROMPTS.map((item) => {
          const Icon = EXAMPLE_ICONS[item.id as keyof typeof EXAMPLE_ICONS] ?? FolderOpen;
          return (
            <button
              key={item.id}
              type="button"
              className={cn(
                "example-card group cursor-pointer rounded-xl border border-slate-200 bg-white p-4 text-left transition-all duration-200",
                "hover:shadow-md dark:border-white/10 dark:bg-[#1A1A1A]"
              )}
              onClick={() => onSelectPrompt(t(item.promptKey))}
            >
              <div className="mb-2 flex items-center gap-2">
                <div
                  className={cn(
                    "flex size-7 items-center justify-center rounded-lg",
                    item.iconClassName
                  )}
                >
                  <Icon className={cn("size-3.5", item.iconColorClassName)} />
                </div>
                <span className="text-sm font-medium text-slate-700 dark:text-slate-200">
                  {t(item.titleKey)}
                </span>
              </div>
              <p className="text-sm leading-relaxed text-slate-500 dark:text-slate-400">
                {t(item.textKey)}
              </p>
            </button>
          );
        })}
      </div>
    </div>
  );
}
