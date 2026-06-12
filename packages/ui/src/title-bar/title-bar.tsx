"use client";

import { Dropdown, type MenuProps } from "antd";
import {
  ArrowUpCircle,
  CircleHelp,
  Copy,
  Headphones,
  Info,
  KeyRound,
  Mail,
  Menu,
  Minus,
  Square,
  X,
  type LucideIcon
} from "lucide-react";
import { memo, useMemo, useState } from "react";

import { cn } from "@supportflow/shared";
import { mainWindow } from "@supportflow/shared/tauri-bridge/window/main-window";
import { Button } from "@supportflow/ui/button";

import { LicenseActivationModal, LicenseMachineCodeModal } from "./license-modals";

export type TitleBarAccent = {
  logoGradient: string;
  title: string;
  barClassName: string;
  logoText: string;
  titleClassName?: string;
  controlClassName?: string;
};

const INTERACTIVE_TITLE_BAR_SELECTOR =
  "button, a, input, select, textarea, [role='menuitem'], .ant-dropdown";

type MoreMenuItem = {
  id: string;
  label: string;
  Icon: LucideIcon;
};

const MORE_MENU_ITEMS: MoreMenuItem[] = [
  { id: "feedback", label: "反馈", Icon: Mail },
  { id: "contact_support", label: "联系支持", Icon: Headphones },
  { id: "online_help", label: "在线帮助", Icon: CircleHelp },
  { id: "check_updates", label: "检查更新", Icon: ArrowUpCircle },
  { id: "about", label: "关于", Icon: Info }
];

function handleTitleBarMouseDown(e: React.MouseEvent<HTMLDivElement>) {
  if (e.buttons !== 1) return;
  const target = e.target as HTMLElement;
  if (target.closest(INTERACTIVE_TITLE_BAR_SELECTOR)) return;
  void mainWindow.startDragging();
}

const TitleBar = memo((props: { height?: number; accent?: TitleBarAccent }) => {
  const h = props.height ?? 40;
  const accent = props.accent;
  const title = accent?.title ?? "SupportFlow";
  const logoText = accent?.logoText ?? "T";
  const logoGradient = accent?.logoGradient ?? "from-[#2b7fff] to-[#155dfc]";

  const controlBtnClass = cn(accent?.controlClassName ?? "text-muted-foreground", "shrink-0");
  const [activationOpen, setActivationOpen] = useState(false);
  const [machineCodeOpen, setMachineCodeOpen] = useState(false);

  const menuItems = useMemo<MenuProps["items"]>(
    () => [
      {
        key: "license_activation",
        label: "订阅激活",
        icon: <KeyRound className="size-4 shrink-0" aria-hidden />,
        onClick: () => setActivationOpen(true)
      },
      {
        key: "license_machine_code",
        label: "机器码",
        icon: <Copy className="size-4 shrink-0" aria-hidden />,
        onClick: () => setMachineCodeOpen(true)
      },
      { type: "divider" as const },
      ...MORE_MENU_ITEMS.map(({ id, label, Icon }) => ({
        key: id,
        label,
        icon: <Icon className="size-4 shrink-0" aria-hidden />,
        disabled: true
      }))
    ],
    []
  );

  return (
    <div
      role="banner"
      data-tauri-drag-region
      className={cn(
        "flex w-full cursor-default items-center justify-between px-3 select-none",
        accent ? accent.barClassName : "bg-card/90 backdrop-blur"
      )}
      style={{ height: h }}
      onMouseDown={handleTitleBarMouseDown}
    >
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <div
          className={cn(
            "flex size-8 shrink-0 items-center justify-center rounded-lg bg-linear-to-br text-sm font-bold text-white",
            logoGradient
          )}
          aria-hidden
        >
          {logoText}
        </div>
        <span
          className={cn(
            "truncate text-[15px] font-semibold tracking-tight",
            accent?.titleClassName ?? (accent ? "text-slate-800" : "text-foreground")
          )}
        >
          {title}
        </span>
      </div>

      <div className="flex shrink-0 items-center gap-2" onMouseDown={(e) => e.stopPropagation()}>
        <div className="ml-1 flex items-center gap-0.5">
          <Dropdown trigger={["click"]} placement="bottomRight" menu={{ items: menuItems }}>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className={controlBtnClass}
              aria-label="菜单"
            >
              <Menu className="size-4" />
            </Button>
          </Dropdown>

          <LicenseActivationModal open={activationOpen} onOpenChange={setActivationOpen} />
          <LicenseMachineCodeModal open={machineCodeOpen} onOpenChange={setMachineCodeOpen} />

          <Button
            type="button"
            variant="ghost"
            size="icon"
            className={controlBtnClass}
            aria-label="最小化"
            onClick={() => void mainWindow.minimize()}
          >
            <Minus className="size-4" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className={controlBtnClass}
            aria-label="最大化"
            onClick={() => void mainWindow.toggleMaximize()}
          >
            <Square className="size-3.5" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className={cn(
              controlBtnClass,
              accent?.controlClassName
                ? "hover:bg-red-500/10 hover:text-red-600"
                : "hover:bg-destructive/10 hover:text-destructive"
            )}
            aria-label="关闭"
            onClick={() => void mainWindow.close()}
          >
            <X className="size-4" />
          </Button>
        </div>
      </div>
    </div>
  );
});

TitleBar.displayName = "TitleBar";

export default TitleBar;
