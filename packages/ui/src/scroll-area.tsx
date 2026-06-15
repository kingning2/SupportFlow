"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import * as React from "react";

import { cn } from "@supportflow/shared";

const ScrollArea = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, children, ...props }, ref) => (
    <div ref={ref} className={cn("relative overflow-auto", className)} {...props}>
      {children}
    </div>
  )
);
ScrollArea.displayName = "ScrollArea";

const ScrollBar = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { orientation?: "horizontal" | "vertical" }
>(({ className, orientation = "vertical", ...props }, ref) => (
  <div
    ref={ref}
    data-orientation={orientation}
    className={cn("hidden", className)}
    aria-hidden
    {...props}
  />
));
ScrollBar.displayName = "ScrollBar";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export { ScrollArea, ScrollBar };
