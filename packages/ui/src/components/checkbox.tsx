"use client";

import { Checkbox as AntdCheckbox, type CheckboxProps as AntdCheckboxProps } from "antd";
import * as React from "react";

import { cn } from "@supportflow/shared";

export interface CheckboxProps extends AntdCheckboxProps {}

const Checkbox = ({ className, ...props }: CheckboxProps) => {
  return <AntdCheckbox className={cn(className)} {...props} />;
};
Checkbox.displayName = "Checkbox";

export { Checkbox };
