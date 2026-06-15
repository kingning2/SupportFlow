"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Tag } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

function mapVariant(variant: BadgeVariant = "default") {
  switch (variant) {
    case "destructive":
      return { color: "red" as const, type: "light" as const };
    case "secondary":
      return { color: "grey" as const, type: "ghost" as const };
    case "outline":
      return { color: "grey" as const, type: "ghost" as const };
    default:
      return { color: "blue" as const, type: "light" as const };
  }
}

function Badge({ className, variant = "default", children, ...props }: BadgeProps) {
  const mapped = mapVariant(variant);

  return (
    <Tag
      {...mapped}
      className={cn("inline-flex items-center", variant === "outline" && "border", className)}
      {...(props as React.ComponentProps<typeof Tag>)}
    >
      {children}
    </Tag>
  );
}

/** @deprecated shadcn cva helper �?kept for import compatibility. */
const badgeVariants = () => "";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export { Badge, badgeVariants };
