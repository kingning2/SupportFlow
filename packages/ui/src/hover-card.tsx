"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Popover } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

type HoverCardProps = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  openDelay?: number;
  closeDelay?: number;
  children: React.ReactNode;
};

const HoverCard = ({ children }: HoverCardProps) => {
  let trigger: React.ReactNode = null;
  let content: React.ReactNode = null;
  let contentClassName: string | undefined;

  React.Children.forEach(children, (child) => {
    if (!React.isValidElement(child)) return;
    if (child.type === HoverCardTrigger) {
      trigger = (child.props as { children: React.ReactNode }).children;
    }
    if (child.type === HoverCardContent) {
      const props = child.props as React.HTMLAttributes<HTMLDivElement>;
      content = props.children;
      contentClassName = props.className;
    }
  });

  if (!trigger) {
    return <>{children}</>;
  }

  return (
    <Popover
      trigger="hover"
      content={<div className={cn(contentClassName)}>{content}</div>}
      mouseEnterDelay={0.2}
    >
      <span className="inline-flex">{trigger}</span>
    </Popover>
  );
};

const HoverCardTrigger = ({ children }: { children: React.ReactNode }) => <>{children}</>;

const HoverCardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { align?: string; sideOffset?: number }
>(({ className, children, align: _align, sideOffset: _sideOffset, ...props }, ref) => (
  <div ref={ref} className={cn("hidden", className)} {...props}>
    {children}
  </div>
));
HoverCardContent.displayName = "HoverCardContent";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export { HoverCard, HoverCardTrigger, HoverCardContent };
