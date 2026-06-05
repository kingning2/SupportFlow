"use client";

import { Tag } from "antd";
import * as React from "react";

import { cn } from "@supportflow/shared";

type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
}

function mapVariant(variant: BadgeVariant = "default") {
  switch (variant) {
    case "destructive":
      return "error" as const;
    case "secondary":
      return "default" as const;
    case "outline":
      return "default" as const;
    default:
      return "processing" as const;
  }
}

function Badge({ className, variant = "default", children, ...props }: BadgeProps) {
  return (
    <Tag
      bordered={variant === "outline"}
      color={mapVariant(variant)}
      className={cn("inline-flex items-center", className)}
      {...(props as React.ComponentProps<typeof Tag>)}
    >
      {children}
    </Tag>
  );
}

/** @deprecated shadcn cva helper — kept for import compatibility. */
const badgeVariants = () => "";

export { Badge, badgeVariants };
