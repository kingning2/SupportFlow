"use client";

import { Spin } from "antd";

import { cn } from "@supportflow/shared";

function Spinner({ className, ...props }: React.ComponentProps<"span">) {
  return (
    <Spin
      size="small"
      className={cn(className)}
      {...(props as React.ComponentProps<typeof Spin>)}
    />
  );
}

export { Spinner };
