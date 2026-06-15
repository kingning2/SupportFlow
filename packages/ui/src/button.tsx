"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Button as SemiButton } from "@douyinfe/semi-ui-19";
import type { ButtonProps as SemiButtonProps } from "@douyinfe/semi-ui-19/lib/es/button";
import * as React from "react";

import { cn } from "@supportflow/shared";

type ButtonVariant = "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
type ButtonSize = "default" | "sm" | "lg" | "icon" | "icon-sm";

/** @deprecated Use `Button` from `@douyinfe/semi-ui-19`. */
export interface ButtonProps extends Omit<
  SemiButtonProps,
  "type" | "size" | "theme" | "htmlType" | "block"
> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  asChild?: boolean;
  type?: "button" | "submit" | "reset";
}

function mapVariant(variant: ButtonVariant = "default"): Pick<SemiButtonProps, "theme" | "type"> {
  switch (variant) {
    case "destructive":
      return { theme: "solid", type: "danger" };
    case "outline":
      return { theme: "light", type: "tertiary" };
    case "secondary":
      return { theme: "light", type: "secondary" };
    case "ghost":
    case "link":
      return { theme: "borderless", type: "tertiary" };
    default:
      return { theme: "solid", type: "primary" };
  }
}

function mapSize(size: ButtonSize = "default"): SemiButtonProps["size"] {
  switch (size) {
    case "sm":
    case "icon-sm":
      return "small";
    case "lg":
      return "large";
    default:
      return "default";
  }
}

const iconSizeClass: Record<ButtonSize, string | undefined> = {
  default: undefined,
  sm: undefined,
  lg: undefined,
  icon: "!size-9 min-w-9 p-0",
  "icon-sm": "!size-8 min-w-8 p-0"
};

/** @deprecated Use `Button` from `@douyinfe/semi-ui-19`. */
const Button = React.forwardRef<React.ElementRef<typeof SemiButton>, ButtonProps>(
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
    const semiSize = mapSize(size);

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
      <SemiButton
        ref={ref}
        htmlType={type}
        size={semiSize}
        className={cn(iconSizeClass[size], className)}
        {...mapped}
        {...props}
      >
        {children}
      </SemiButton>
    );
  }
);
Button.displayName = "Button";

/** @deprecated Unused shadcn helper; use Semi `Button` theme/type props instead. */
const buttonVariants = () => "";

/** @deprecated Use `Button` from `@douyinfe/semi-ui-19`. */
export { Button, buttonVariants };
