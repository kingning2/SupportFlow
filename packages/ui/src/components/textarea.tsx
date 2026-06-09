"use client";

import { Input } from "antd";
import * as React from "react";

import { cn } from "@supportflow/shared";

const { TextArea: AntdTextArea } = Input;

const Textarea = ({ className, ...props }: React.ComponentProps<"textarea">) => {
  return (
    <AntdTextArea
      className={cn(className)}
      {...(props as React.ComponentProps<typeof AntdTextArea>)}
    />
  );
};
Textarea.displayName = "Textarea";

export { Textarea };
