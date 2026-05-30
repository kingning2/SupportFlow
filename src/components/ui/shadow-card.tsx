import * as React from "react";

import { cn } from "@/lib/utils";

export type ShadowCardProps = React.HTMLAttributes<HTMLDivElement>;

/**
 * A Card-like container with consistent rounded corners and hover lift + shadow.
 * Designed to avoid "overflow-hidden" clipping during translate/hover.
 */
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
