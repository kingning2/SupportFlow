"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { TextArea } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

const Textarea = ({ className, ...props }: React.ComponentProps<"textarea">) => {
  return (
    <TextArea className={cn(className)} {...(props as React.ComponentProps<typeof TextArea>)} />
  );
};
Textarea.displayName = "Textarea";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */
export { Textarea };
