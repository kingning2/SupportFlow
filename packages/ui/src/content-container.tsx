import * as React from "react";

import { cn } from "@supportflow/shared";

export type ContentContainerProps = React.HTMLAttributes<HTMLDivElement>;

/**
 * 主内容区：在纵向 flex 布局中占满剩余空间（flex-1 + min-h-0），横向同理（min-w-0），
 * 超出部分滚动；滚动条样式由全局 `.scrollbar-themed` 提供。
 */
const ContentContainer = React.forwardRef<HTMLDivElement, ContentContainerProps>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn("scrollbar-themed min-h-0 w-full min-w-0 flex-1 overflow-auto", className)}
      {...props}
    />
  )
);
ContentContainer.displayName = "ContentContainer";

export { ContentContainer };
