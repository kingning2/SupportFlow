"use client";

import { Input as AntdInput } from "antd";
import * as React from "react";

import { cn } from "@supportflow/shared";

const Input = ({ className, type, ...props }: React.ComponentProps<"input">) => {
  return (
    <AntdInput
      type={type}
      className={cn(className)}
      {...(props as React.ComponentProps<typeof AntdInput>)}
    />
  );
};
Input.displayName = "Input";

export { Input };
