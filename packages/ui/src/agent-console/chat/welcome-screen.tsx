"use client";

import { BookOpen, Clock, Code, FolderOpen, Globe, Zap } from "lucide-react";

import { EXAMPLE_PROMPTS } from "../constants/example-prompts";
import { ConsoleBrandMark } from "../shared/console-brand";
import { cn } from "@supportflow/shared";
import { Button } from "@supportflow/ui/button";

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
  return (
    <div
      className="flex h-full flex-col items-center justify-center px-6 pb-16"
      style={{ paddingTop: "6vh" }}
    >
      <ConsoleBrandMark className="mb-6 size-16" />
      <h1 className="text-foreground mb-3 text-2xl font-bold">SupportFlow</h1>
      <p className="text-muted-foreground mb-10 max-w-lg text-center leading-relaxed">
        我可以帮你梳理问题、修改代码、管理知识、使用技能工具，并持续积累上下文。
      </p>

      <div className="grid w-full max-w-2xl grid-cols-2 gap-3 sm:grid-cols-3">
        {EXAMPLE_PROMPTS.map((item) => {
          const Icon = EXAMPLE_ICONS[item.id as keyof typeof EXAMPLE_ICONS] ?? FolderOpen;
          return (
            <Button
              key={item.id}
              type="button"
              variant="outline"
              className={cn(
                "example-card group bg-card h-auto cursor-pointer rounded-xl p-4 text-left transition-all duration-200 hover:shadow-md"
              )}
              onClick={() => onSelectPrompt(item.prompt)}
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
                <span className="text-foreground text-sm font-medium">{item.title}</span>
              </div>
              <p className="text-muted-foreground text-sm leading-relaxed">{item.text}</p>
            </Button>
          );
        })}
      </div>
    </div>
  );
}
