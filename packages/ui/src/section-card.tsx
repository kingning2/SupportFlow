"use client";

import * as React from "react";

import { cn } from "@supportflow/shared";

export interface SectionCardProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 卡片标题 */
  title?: string;
  /** 标题右侧操作 */
  extra?: React.ReactNode;
  /** 内边距大小，默认 "md" */
  padding?: "sm" | "md" | "lg" | "none";
}

const paddingMap: Record<NonNullable<SectionCardProps["padding"]>, string> = {
  none: "",
  sm: "p-3",
  md: "p-4",
  lg: "p-6"
};

/**
 * 分区卡片：圆角卡片容器，带可选标题栏。
 * 用于页面内按功能分区展示。
 *
 * 用法：
 * ```tsx
 * <SectionCard title="API 配置" extra={<Button size="sm">保存</Button>}>
 *   ...表单内容...
 * </SectionCard>
 * ```
 */
const SectionCard = React.forwardRef<HTMLDivElement, SectionCardProps>(
  ({ title, extra, padding = "md", className, children, ...props }, ref) => (
    <div
      ref={ref}
      className={cn("bg-card border-border rounded-2xl border shadow-sm", className)}
      {...props}
    >
      {title ? (
        <div className="border-border/60 flex items-center justify-between border-b px-4 py-3">
          <h3 className="text-foreground text-sm font-semibold">{title}</h3>
          {extra ? <div className="flex items-center gap-2">{extra}</div> : null}
        </div>
      ) : null}
      <div className={paddingMap[padding]}>{children}</div>
    </div>
  )
);
SectionCard.displayName = "SectionCard";

export { SectionCard };
