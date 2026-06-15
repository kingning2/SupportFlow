"use client";

/**
 * @deprecated Import from `@douyinfe/semi-ui-19` instead.
 *             This `@supportflow/ui/*` path is a compatibility shim only.
 */
import { Dropdown } from "@douyinfe/semi-ui-19";
import * as React from "react";

import { cn } from "@supportflow/shared";

type MenuItemEntry = {
  key: string;
  type?: "divider" | "submenu";
  label?: React.ReactNode;
  disabled?: boolean;
  onClick?: () => void;
  children?: MenuItemEntry[];
};

type DropdownMenuContextValue = {
  registerItem: (item: MenuItemEntry) => () => void;
  setTrigger: (node: React.ReactNode) => void;
  setContentClassName: (className?: string) => void;
  setAlign: (align?: "start" | "center" | "end") => void;
  setSideOffset: (offset?: number) => void;
  registerExtra: (node: React.ReactNode) => () => void;
};

const DropdownMenuContext = React.createContext<DropdownMenuContextValue | null>(null);

function useDropdownMenuContext() {
  const ctx = React.useContext(DropdownMenuContext);
  if (!ctx) {
    throw new Error("DropdownMenu compound components must be used within DropdownMenu");
  }
  return ctx;
}

function renderMenuItems(items: MenuItemEntry[]): React.ReactNode {
  return items.map((item) => {
    if (item.type === "divider") {
      return <Dropdown.Divider key={item.key} />;
    }

    if (item.children && item.children.length > 0) {
      return (
        <Dropdown
          key={item.key}
          position="rightTop"
          trigger="hover"
          render={<Dropdown.Menu>{renderMenuItems(item.children)}</Dropdown.Menu>}
        >
          <Dropdown.Item disabled={item.disabled}>{item.label}</Dropdown.Item>
        </Dropdown>
      );
    }

    return (
      <Dropdown.Item key={item.key} disabled={item.disabled} onClick={item.onClick}>
        {item.label}
      </Dropdown.Item>
    );
  });
}

type DropdownMenuProps = {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children: React.ReactNode;
};

const DropdownMenu = ({ open, onOpenChange, children }: DropdownMenuProps) => {
  const [trigger, setTrigger] = React.useState<React.ReactNode>(null);
  const [items, setItems] = React.useState<MenuItemEntry[]>([]);
  const [align, setAlignState] = React.useState<"start" | "center" | "end">("center");
  const [sideOffset, setSideOffsetState] = React.useState(4);
  const [extraNodes, setExtraNodes] = React.useState<React.ReactNode[]>([]);
  const [contentClassNameState, setContentClassNameState] = React.useState<string>();

  const setAlign = React.useCallback((next?: "start" | "center" | "end") => {
    if (next) setAlignState(next);
  }, []);

  const setSideOffset = React.useCallback((next?: number) => {
    if (next !== undefined) setSideOffsetState(next);
  }, []);

  const setContentClassName = React.useCallback((next?: string) => {
    setContentClassNameState(next);
  }, []);

  const registerItem = React.useCallback((item: MenuItemEntry) => {
    setItems((prev) => [...prev, item]);
    return () => setItems((prev) => prev.filter((entry) => entry !== item));
  }, []);

  const registerExtra = React.useCallback((node: React.ReactNode) => {
    setExtraNodes((prev) => [...prev, node]);
    return () => setExtraNodes((prev) => prev.filter((entry) => entry !== node));
  }, []);

  const ctx = React.useMemo(
    () => ({
      registerItem,
      setTrigger,
      setContentClassName,
      setAlign,
      setSideOffset,
      registerExtra
    }),
    [registerItem, registerExtra, setAlign, setSideOffset, setContentClassName]
  );

  const position = align === "start" ? "bottomLeft" : align === "end" ? "bottomRight" : "bottom";

  const menu = (
    <Dropdown.Menu className={cn(contentClassNameState)}>
      {renderMenuItems(items)}
      {extraNodes}
    </Dropdown.Menu>
  );

  return (
    <DropdownMenuContext.Provider value={ctx}>
      <div className="contents">{children}</div>
      {trigger ? (
        <Dropdown
          visible={open}
          onVisibleChange={onOpenChange}
          trigger="click"
          position={position}
          spacing={sideOffset}
          render={menu}
        >
          <span className="inline-flex">{trigger}</span>
        </Dropdown>
      ) : null}
    </DropdownMenuContext.Provider>
  );
};

const DropdownMenuTrigger = ({
  asChild,
  children
}: {
  asChild?: boolean;
  children: React.ReactElement;
}) => {
  const { setTrigger } = useDropdownMenuContext();
  const node = asChild ? children : children;

  React.useLayoutEffect(() => {
    setTrigger(node);
    return () => setTrigger(null);
  }, [node, setTrigger]);

  return null;
};

const DropdownMenuContent = ({
  className,
  align,
  sideOffset,
  children,
  onPointerDown
}: {
  className?: string;
  align?: "start" | "center" | "end";
  sideOffset?: number;
  children: React.ReactNode;
  onPointerDown?: (e: React.PointerEvent) => void;
}) => {
  const { setContentClassName, setAlign, setSideOffset, registerExtra } = useDropdownMenuContext();

  React.useLayoutEffect(() => {
    setContentClassName(className);
    if (align) setAlign(align);
    if (sideOffset !== undefined) setSideOffset(sideOffset);
  }, [className, align, sideOffset, setContentClassName, setAlign, setSideOffset]);

  React.useLayoutEffect(() => {
    if (!onPointerDown) return;
    return registerExtra(<div className="hidden" onPointerDown={onPointerDown} aria-hidden />);
  }, [onPointerDown, registerExtra]);

  return <>{children}</>;
};

type SubMenuContextValue = {
  registerSubItem: (item: MenuItemEntry) => () => void;
};

const SubMenuContext = React.createContext<SubMenuContextValue | null>(null);

let dropdownItemCounter = 0;

const DropdownMenuItem = React.forwardRef<
  HTMLDivElement,
  Omit<React.HTMLAttributes<HTMLDivElement>, "onSelect"> & {
    inset?: boolean;
    disabled?: boolean;
    onSelect?: (event: Event) => void;
  }
>(({ className, children, disabled, onSelect, onClick, ...props }, ref) => {
  const sub = React.useContext(SubMenuContext);
  const { registerItem } = useDropdownMenuContext();
  const register = sub?.registerSubItem ?? registerItem;
  const itemKey = React.useMemo(() => `dropdown-item-${++dropdownItemCounter}`, []);

  React.useLayoutEffect(() => {
    const item: MenuItemEntry = {
      key: itemKey,
      label: <span className={cn(className)}>{children}</span>,
      disabled,
      onClick: () => {
        onClick?.({} as React.MouseEvent<HTMLDivElement>);
        onSelect?.(new Event("select"));
      }
    };
    return register(item);
  }, [register, itemKey, className, children, disabled, onClick, onSelect]);

  return <div ref={ref} hidden {...props} />;
});
DropdownMenuItem.displayName = "DropdownMenuItem";

const DropdownMenuCheckboxItem = DropdownMenuItem;
const DropdownMenuRadioItem = DropdownMenuItem;

const DropdownMenuLabel = ({
  children,
  className
}: {
  children: React.ReactNode;
  className?: string;
  inset?: boolean;
}) => (
  <div className={cn("px-2 py-1.5 text-sm font-semibold", className)} hidden>
    {children}
  </div>
);

const DropdownMenuSeparator = () => {
  const { registerItem } = useDropdownMenuContext();
  const key = React.useMemo(() => `dropdown-divider-${++dropdownItemCounter}`, []);

  React.useLayoutEffect(() => {
    return registerItem({ type: "divider", key });
  }, [registerItem, key]);

  return null;
};

const DropdownMenuShortcut = ({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) => (
  <span className={cn("ml-auto text-xs tracking-widest opacity-60", className)} {...props} />
);

const DropdownMenuGroup = ({ children }: { children: React.ReactNode }) => <>{children}</>;
const DropdownMenuPortal = ({ children }: { children: React.ReactNode }) => <>{children}</>;
const DropdownMenuRadioGroup = ({ children }: { children: React.ReactNode }) => <>{children}</>;

const DropdownMenuSub = ({ children }: { children: React.ReactNode }) => <>{children}</>;

const DropdownMenuSubTrigger = ({
  children,
  className
}: {
  children: React.ReactNode;
  className?: string;
  inset?: boolean;
}) => {
  const parent = useDropdownMenuContext();
  const subKey = React.useMemo(() => `dropdown-sub-${++dropdownItemCounter}`, []);
  const [subItems, setSubItems] = React.useState<MenuItemEntry[]>([]);

  const registerSubItem = React.useCallback((item: MenuItemEntry) => {
    setSubItems((prev) => [...prev, item]);
    return () => setSubItems((prev) => prev.filter((entry) => entry !== item));
  }, []);

  React.useLayoutEffect(() => {
    if (subItems.length === 0) return;
    return parent.registerItem({
      key: subKey,
      type: "submenu",
      label: <span className={cn(className)}>{children}</span>,
      children: subItems
    });
  }, [parent, subKey, className, children, subItems]);

  return <SubMenuContext.Provider value={{ registerSubItem }}>{children}</SubMenuContext.Provider>;
};

const DropdownMenuSubContent = ({ children }: { children: React.ReactNode }) => <>{children}</>;

/** @deprecated Use Semi components from `@douyinfe/semi-ui-19` instead. */

export {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuGroup,
  DropdownMenuPortal,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuRadioGroup
};
