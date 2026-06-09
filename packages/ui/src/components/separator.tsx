"use client";

import { Divider } from "antd";
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
    <Divider type={orientation === "vertical" ? "vertical" : "horizontal"} />
  </div>
));
Separator.displayName = "Separator";

export { Separator };
