"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Checkbox as SemiCheckbox } from "@douyinfe/semi-ui-19";
import type { CheckboxProps as SemiCheckboxProps } from "@douyinfe/semi-ui-19/lib/es/checkbox";
import * as React from "react";

import { cn } from "@supportflow/shared";

export interface CheckboxProps extends SemiCheckboxProps {}

const Checkbox = ({ className, ...props }: CheckboxProps) => {
  return <SemiCheckbox className={cn(className)} {...props} />;
};
Checkbox.displayName = "Checkbox";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */
export { Checkbox };
