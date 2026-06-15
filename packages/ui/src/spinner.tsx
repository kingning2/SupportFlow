"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Spin } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

function Spinner({ className, ...props }: React.ComponentProps<"span">) {
  return <Spin size="small" className={cn(className)} {...(props as object)} />;
}

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */
export { Spinner };
