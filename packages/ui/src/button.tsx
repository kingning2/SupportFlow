"use client";

import { Button as AntdButton, type ButtonProps as AntdButtonProps } from "antd";
import * as React from "react";

import { cn } from "@supportflow/shared";

type ButtonVariant = "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
type ButtonSize = "default" | "sm" | "lg" | "icon" | "icon-sm";

export interface ButtonProps extends Omit<
  AntdButtonProps,
  "type" | "size" | "variant" | "htmlType"
> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  asChild?: boolean;
  /** HTML button type attribute */
  type?: "button" | "submit" | "reset";
}

function mapVariant(variant: ButtonVariant = "default"): Pick<AntdButtonProps, "type" | "danger"> {
  switch (variant) {
    case "destructive":
      return { type: "primary", danger: true };
    case "outline":
    case "secondary":
      return { type: "default" };
    case "ghost":
      return { type: "text" };
    case "link":
      return { type: "link" };
    default:
      return { type: "primary" };
  }
}

function mapSize(size: ButtonSize = "default"): AntdButtonProps["size"] {
  switch (size) {
    case "sm":
    case "icon-sm":
      return "small";
    case "lg":
      return "large";
    default:
      return "middle";
  }
}

const iconSizeClass: Record<ButtonSize, string | undefined> = {
  default: undefined,
  sm: undefined,
  lg: undefined,
  icon: "!size-9 min-w-9 p-0",
  "icon-sm": "!size-8 min-w-8 p-0"
};

const Button = React.forwardRef<HTMLButtonElement | HTMLAnchorElement, ButtonProps>(
  (
    {
      className,
      variant = "default",
      size = "default",
      asChild = false,
      type = "button",
      children,
      ...props
    },
    ref
  ) => {
    const mapped = mapVariant(variant);
    const antdSize = mapSize(size);

    if (asChild && React.isValidElement(children)) {
      return React.cloneElement(
        children as React.ReactElement<{ className?: string; ref?: unknown }>,
        {
          className: cn(
            (children as React.ReactElement<{ className?: string }>).props.className,
            iconSizeClass[size],
            className
          ),
          ref
        }
      );
    }

    return (
      <AntdButton
        ref={ref}
        htmlType={type}
        size={antdSize}
        className={cn(iconSizeClass[size], className)}
        {...mapped}
        {...props}
      >
        {children}
      </AntdButton>
    );
  }
);
Button.displayName = "Button";

/** @deprecated shadcn cva helper — kept for import compatibility. */
const buttonVariants = () => "";

export { Button, buttonVariants };
