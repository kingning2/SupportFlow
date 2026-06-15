"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import * as React from "react";

import { cn } from "@supportflow/shared";
import { Button } from "./button";

type CollapsibleContextValue = {
  open: boolean;
  setOpen: (open: boolean) => void;
};

const CollapsibleContext = React.createContext<CollapsibleContextValue | null>(null);

type CollapsibleProps = {
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  disabled?: boolean;
  className?: string;
  children: React.ReactNode;
};

const Collapsible = ({
  open: openProp,
  defaultOpen = false,
  onOpenChange,
  className,
  children
}: CollapsibleProps) => {
  const [internalOpen, setInternalOpen] = React.useState(defaultOpen);
  const open = openProp ?? internalOpen;

  const setOpen = React.useCallback(
    (next: boolean) => {
      onOpenChange?.(next);
      if (openProp === undefined) {
        setInternalOpen(next);
      }
    },
    [onOpenChange, openProp]
  );

  return (
    <CollapsibleContext.Provider value={{ open, setOpen }}>
      <div className={cn(className)}>{children}</div>
    </CollapsibleContext.Provider>
  );
};

const CollapsibleTrigger = React.forwardRef<
  React.ElementRef<typeof Button>,
  React.ComponentPropsWithoutRef<typeof Button> & { asChild?: boolean }
>(({ asChild, children, onClick, ...props }, ref) => {
  const ctx = React.useContext(CollapsibleContext);
  if (!ctx) {
    throw new Error("CollapsibleTrigger must be used within Collapsible");
  }

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    onClick?.(event as React.MouseEvent<HTMLButtonElement>);
    if (!event.defaultPrevented) {
      ctx.setOpen(!ctx.open);
    }
  };

  if (asChild && React.isValidElement(children)) {
    return React.cloneElement(children as React.ReactElement<{ onClick?: typeof handleClick }>, {
      onClick: handleClick
    });
  }

  return (
    <Button ref={ref} type="button" onClick={handleClick} {...props}>
      {children}
    </Button>
  );
});
CollapsibleTrigger.displayName = "CollapsibleTrigger";

const CollapsibleContent = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, children, ...props }, ref) => {
    const ctx = React.useContext(CollapsibleContext);
    if (!ctx?.open) {
      return null;
    }

    return (
      <div ref={ref} className={cn(className)} {...props}>
        {children}
      </div>
    );
  }
);
CollapsibleContent.displayName = "CollapsibleContent";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export { Collapsible, CollapsibleTrigger, CollapsibleContent };
