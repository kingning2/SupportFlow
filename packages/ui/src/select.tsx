"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Select as SemiSelect } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

type SelectOption = {
  value: string;
  label: React.ReactNode;
  disabled?: boolean;
};

type SelectContextValue = {
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  options: SelectOption[];
  registerOption: (option: SelectOption) => () => void;
  placeholder?: string;
  setPlaceholder: (placeholder?: string) => void;
  triggerClassName?: string;
  setTriggerClassName: (className?: string) => void;
  contentClassName?: string;
  setContentClassName: (className?: string) => void;
};

const SelectContext = React.createContext<SelectContextValue | null>(null);

function useSelectContext() {
  const ctx = React.useContext(SelectContext);
  if (!ctx) {
    throw new Error("Select compound components must be used within Select");
  }
  return ctx;
}

type SelectProps = {
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  children: React.ReactNode;
};

const Select = ({ value, onValueChange, disabled, children }: SelectProps) => {
  const [options, setOptions] = React.useState<SelectOption[]>([]);
  const [placeholder, setPlaceholder] = React.useState<string>();
  const [triggerClassName, setTriggerClassName] = React.useState<string>();
  const [contentClassName, setContentClassName] = React.useState<string>();

  const registerOption = React.useCallback((option: SelectOption) => {
    setOptions((prev) => {
      const idx = prev.findIndex((item) => item.value === option.value);
      if (idx === -1) {
        return [...prev, option];
      }
      const next = [...prev];
      next[idx] = option;
      return next;
    });
    return () => {
      setOptions((prev) => prev.filter((item) => item.value !== option.value));
    };
  }, []);

  const ctx = React.useMemo(
    () => ({
      value,
      onValueChange,
      disabled,
      options,
      registerOption,
      placeholder,
      setPlaceholder,
      triggerClassName,
      setTriggerClassName,
      contentClassName,
      setContentClassName
    }),
    [
      value,
      onValueChange,
      disabled,
      options,
      registerOption,
      placeholder,
      triggerClassName,
      contentClassName
    ]
  );

  return <SelectContext.Provider value={ctx}>{children}</SelectContext.Provider>;
};

const SelectGroup = ({ children }: { children: React.ReactNode }) => <>{children}</>;

const SelectValue = ({ placeholder }: { placeholder?: string; className?: string }) => {
  const { setPlaceholder } = useSelectContext();
  React.useLayoutEffect(() => {
    setPlaceholder(placeholder);
  }, [placeholder, setPlaceholder]);
  return null;
};

const SelectTrigger = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement> & { children?: React.ReactNode }
>(({ className, children, ...props }, _ref) => {
  const ctx = useSelectContext();

  React.useLayoutEffect(() => {
    ctx.setTriggerClassName(className);
  }, [className, ctx]);

  React.Children.forEach(children, (child) => {
    if (React.isValidElement(child) && child.type === SelectValue) {
      const p = (child.props as { placeholder?: string }).placeholder;
      ctx.setPlaceholder(p);
    }
  });

  return (
    <SemiSelect
      value={ctx.value}
      onChange={(value) => ctx.onValueChange?.(String(value))}
      placeholder={ctx.placeholder}
      disabled={ctx.disabled}
      className={cn("w-full", ctx.triggerClassName)}
      dropdownClassName={ctx.contentClassName}
      optionList={ctx.options.map((option) => ({
        value: option.value,
        label: option.label,
        disabled: option.disabled
      }))}
      {...(props as React.ComponentProps<typeof SemiSelect>)}
    />
  );
});
SelectTrigger.displayName = "SelectTrigger";

const SelectContent = ({
  className,
  children,
  align: _align,
  position: _position
}: {
  className?: string;
  children: React.ReactNode;
  align?: string;
  position?: string;
}) => {
  const { setContentClassName } = useSelectContext();
  React.useLayoutEffect(() => {
    setContentClassName(className);
  }, [className, setContentClassName]);
  return <>{children}</>;
};

const SelectLabel = ({
  children,
  className
}: {
  children: React.ReactNode;
  className?: string;
}) => <div className={cn("px-2 py-1.5 text-xs font-semibold", className)}>{children}</div>;

const SelectItem = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & { value: string; disabled?: boolean }
>(({ value, children, disabled, className, ...props }, ref) => {
  const { registerOption } = useSelectContext();

  React.useLayoutEffect(() => {
    return registerOption({ value, label: children, disabled });
  }, [registerOption, value, children, disabled]);

  return (
    <div ref={ref} className={className} data-value={value} hidden {...props}>
      {children}
    </div>
  );
});
SelectItem.displayName = "SelectItem";

const SelectSeparator = () => null;
const SelectScrollUpButton = () => null;
const SelectScrollDownButton = () => null;

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export {
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
  SelectScrollUpButton,
  SelectScrollDownButton
};
