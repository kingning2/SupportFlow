"use client";

import * as React from "react";

import { cn } from "@supportflow/shared";

export interface PageHeaderProps {
  title: string;
  description?: string;
  children?: React.ReactNode;
  className?: string;
}

/**
 * 页面标题栏：标题 + 描述，右侧透传 children 作为操作区。
 * 适用于不需要完整 PageContainer 的场景（如在已有 flex 布局中嵌入）。
 */
export function PageHeader({ title, description, children, className }: PageHeaderProps) {
  return (
    <div
      className={cn(
        "border-border/70 bg-card/88 flex flex-col gap-3 border-b px-6 py-4 backdrop-blur lg:flex-row lg:items-center lg:justify-between",
        className
      )}
    >
      <div className="min-w-0">
        <h2 className="text-foreground text-base font-semibold">{title}</h2>
        {description ? <p className="text-muted-foreground mt-1 text-sm">{description}</p> : null}
      </div>
      {children ? <div className="flex items-center gap-2">{children}</div> : null}
    </div>
  );
}
