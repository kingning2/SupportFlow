"use client";

import * as React from "react";

import { cn } from "@supportflow/shared";

export interface PageContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 页面标题 */
  title?: string;
  /** 页面描述 */
  description?: string;
  /** 标题右侧操作区 */
  actions?: React.ReactNode;
  /** 是否显示顶部 header（默认 true） */
  header?: boolean;
}

/**
 * 页面级容器：统一处理页面标题 + 描述 + 内容区域滚动。
 * 替代各视图里零散的 `flex h-full min-h-0 flex-col overflow-hidden` 手写模式。
 *
 * 用法：
 * ```tsx
 * <PageContainer title="模型配置" description="管理 API Key 与模型参数">
 *   <SectionCard title="当前模型">...</SectionCard>
 * </PageContainer>
 * ```
 */
const PageContainer = React.forwardRef<HTMLDivElement, PageContainerProps>(
  ({ title, description, actions, header = true, className, children, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "flex h-full min-h-0 flex-col overflow-hidden bg-[var(--main-window-bg,#f7faff)]",
        className
      )}
      {...props}
    >
      {header && (title || description || actions) ? (
        <div className="border-border/70 bg-card/88 shrink-0 border-b px-6 py-4 backdrop-blur">
          <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
            <div className="min-w-0">
              {title ? <h2 className="text-foreground text-base font-semibold">{title}</h2> : null}
              {description ? (
                <p className="text-muted-foreground mt-1 text-sm">{description}</p>
              ) : null}
            </div>
            {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
          </div>
        </div>
      ) : null}
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </div>
  )
);
PageContainer.displayName = "PageContainer";

export { PageContainer };
