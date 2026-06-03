"use client";

import { Tooltip as AntdTooltip } from "antd";
import * as React from "react";

import { cn } from "@supportflow/shared";

const TooltipProvider = ({ children }: { children: React.ReactNode; delayDuration?: number }) => (
  <>{children}</>
);

type TooltipProps = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: React.ReactNode;
};

const Tooltip = ({ children }: TooltipProps) => <>{children}</>;

const TooltipTrigger = ({
  asChild,
  children
}: {
  asChild?: boolean;
  children: React.ReactElement;
}) => (asChild ? children : children);

const TooltipContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    side?: "top" | "right" | "bottom" | "left";
    sideOffset?: number;
  }
>(({ className, side = "top", children, ...props }, ref) => {
  return (
    <div ref={ref} className={cn("hidden", className)} data-side={side} {...props}>
      {children}
    </div>
  );
});
TooltipContent.displayName = "TooltipContent";

type TooltipCompoundProps = {
  children: React.ReactNode;
};

/** Resolves shadcn-style Tooltip + Trigger + Content tree into one antd Tooltip. */
function TooltipCompound({ children }: TooltipCompoundProps) {
  let trigger: React.ReactNode = null;
  let content: React.ReactNode = null;
  let side: "top" | "right" | "bottom" | "left" = "top";

  React.Children.forEach(children, (child) => {
    if (!React.isValidElement(child)) return;
    if (child.type === TooltipTrigger) {
      trigger = (child.props as { children: React.ReactNode }).children;
    }
    if (child.type === TooltipContent) {
      const props = child.props as {
        children: React.ReactNode;
        side?: "top" | "right" | "bottom" | "left";
      };
      content = props.children;
      side = props.side ?? "top";
    }
  });

  if (!trigger) {
    return <>{children}</>;
  }

  return (
    <AntdTooltip title={content} placement={side}>
      <span className="inline-flex">{trigger}</span>
    </AntdTooltip>
  );
}

const TooltipRoot = ({ children, ...props }: TooltipProps) => {
  void props;
  const childArray = React.Children.toArray(children);
  const hasContent = childArray.some(
    (child) => React.isValidElement(child) && child.type === TooltipContent
  );

  if (hasContent) {
    return <TooltipCompound>{children}</TooltipCompound>;
  }

  return <Tooltip>{children}</Tooltip>;
};

export { TooltipRoot as Tooltip, TooltipTrigger, TooltipContent, TooltipProvider };
