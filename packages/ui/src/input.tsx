"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Input as SemiInput } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

const Input = ({ className, type, ...props }: React.ComponentProps<"input">) => {
  return (
    <SemiInput
      type={type}
      className={cn(className)}
      {...(props as React.ComponentProps<typeof SemiInput>)}
    />
  );
};
Input.displayName = "Input";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */
export { Input };
