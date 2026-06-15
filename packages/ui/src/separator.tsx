"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Divider } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

const Separator = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    orientation?: "horizontal" | "vertical";
    decorative?: boolean;
  }
>(({ className, orientation = "horizontal", decorative: _decorative, ...props }, ref) => (
  <div ref={ref} className={cn(className)} {...props}>
    <Divider layout={orientation === "vertical" ? "vertical" : "horizontal"} />
  </div>
));
Separator.displayName = "Separator";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export { Separator };
