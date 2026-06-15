/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import * as React from "react";

import { cn } from "@supportflow/shared";

/** @deprecated Use `Card` from `@douyinfe/semi-ui-19`. */
export type ShadowCardProps = React.HTMLAttributes<HTMLDivElement>;

/**
 * A Card-like container with consistent rounded corners and hover lift + shadow.
 * Designed to avoid "overflow-hidden" clipping during translate/hover.
 */
/** @deprecated Use `Card` from `@douyinfe/semi-ui-19`. */
export const ShadowCard = React.forwardRef<HTMLDivElement, ShadowCardProps>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "shadow-card bg-card text-card-foreground relative z-0 overflow-visible rounded-3xl border",
        "focus-visible:ring-ring transform-gpu focus-visible:ring-2 focus-visible:outline-none",
        className
      )}
      {...props}
    />
  )
);

ShadowCard.displayName = "ShadowCard";
