"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Modal } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

type DialogContextValue = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
};

const DialogContext = React.createContext<DialogContextValue>({});

type DialogProps = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: React.ReactNode;
};

const Dialog = ({ open, onOpenChange, children }: DialogProps) => (
  <DialogContext.Provider value={{ open, onOpenChange }}>{children}</DialogContext.Provider>
);

const DialogTrigger = ({ children }: { children: React.ReactElement }) => children;

const DialogPortal = ({ children }: { children: React.ReactNode }) => <>{children}</>;

const DialogClose = ({ children }: { children?: React.ReactNode }) => <>{children}</>;

const DialogOverlay = () => null;

const DialogContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { onPointerDownOutside?: (e: Event) => void }
>(({ className, children, onPointerDownOutside: _ignored, ...props }, ref) => {
  const { open, onOpenChange } = React.useContext(DialogContext);

  return (
    <Modal
      visible={open}
      onCancel={() => onOpenChange?.(false)}
      footer={null}
      className={cn(className)}
    >
      <div ref={ref} {...props}>
        {children}
      </div>
    </Modal>
  );
});
DialogContent.displayName = "DialogContent";

const DialogHeader = ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
  <div className={cn("mb-4 flex flex-col space-y-1.5", className)} {...props} />
);
DialogHeader.displayName = "DialogHeader";

const DialogFooter = ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
  <div className={cn("mt-4 flex flex-wrap justify-end gap-2", className)} {...props} />
);
DialogFooter.displayName = "DialogFooter";

const DialogTitle = React.forwardRef<HTMLHeadingElement, React.HTMLAttributes<HTMLHeadingElement>>(
  ({ className, ...props }, ref) => (
    <h2 ref={ref} className={cn("text-lg leading-none font-semibold", className)} {...props} />
  )
);
DialogTitle.displayName = "DialogTitle";

const DialogDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p ref={ref} className={cn("text-muted-foreground text-sm", className)} {...props} />
));
DialogDescription.displayName = "DialogDescription";

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export {
  Dialog,
  DialogPortal,
  DialogOverlay,
  DialogTrigger,
  DialogClose,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription
};
