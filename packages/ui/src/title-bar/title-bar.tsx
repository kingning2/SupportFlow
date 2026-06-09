"use client";

import { Dropdown, type MenuProps } from "antd";
import {
  ArrowUpCircle,
  Check,
  CircleHelp,
  Copy,
  Globe,
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
import { memo, useCallback, useMemo, useState } from "react";

import { cn } from "@supportflow/shared";
import { useAppDispatch, useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";
import { changeCurrentLanguageAction } from "@supportflow/shared/desktop-shell/store/modules/app";
import { setLang } from "@supportflow/shared/tauri-bridge/cmd/lang";
import type { Language } from "@supportflow/shared/tauri-bridge/enums";
import { mainWindow } from "@supportflow/shared/tauri-bridge/window/main-window";
import { Button } from "@supportflow/ui/button";

import { LicenseActivationModal, LicenseMachineCodeModal } from "./license-modals";

/** 通道 flavor 可选品牌色；未传时使用默认控制台标题栏样式 */
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
  { id: "feedback", label: "反饋", Icon: Mail },
  { id: "contact_support", label: "聯絡支援", Icon: Headphones },
  { id: "online_help", label: "在線幫助", Icon: CircleHelp },
  { id: "check_updates", label: "檢查更新", Icon: ArrowUpCircle },
  { id: "about", label: "關於", Icon: Info }
];

/** 整栏可拖；仅排除按钮/菜单等可交互控件（由控件区 stopPropagation 兜底） */
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
  const dispatch = useAppDispatch();
  const currentLanguage = useAppSelector((state) => state.app.currentLanguage);
  const supportLanguages = useAppSelector((state) => state.app.supportLanguages);

  const switchLanguage = useCallback(
    async (next: Language) => {
      if (next === currentLanguage) return;
      try {
        await setLang(next);
      } catch {
        dispatch(changeCurrentLanguageAction(next));
      }
    },
    [currentLanguage, dispatch]
  );

  const controlBtnClass = cn(accent?.controlClassName ?? "text-muted-foreground", "shrink-0");
  const [activationOpen, setActivationOpen] = useState(false);
  const [machineCodeOpen, setMachineCodeOpen] = useState(false);

  const menuItems = useMemo<MenuProps["items"]>(
    () => [
      {
        key: "language",
        label: "語言",
        icon: <Globe className="size-4 shrink-0 opacity-80" aria-hidden />,
        children: supportLanguages.map((opt) => ({
          key: opt.value,
          label: (
            <span className="flex items-center gap-2">
              <span className="flex size-4 shrink-0 items-center justify-center">
                {currentLanguage === opt.value ? (
                  <Check className="text-primary size-4" aria-hidden />
                ) : null}
              </span>
              {opt.label}
            </span>
          ),
          onClick: () => {
            void switchLanguage(opt.value);
          }
        }))
      },
      { type: "divider" as const },
      {
        key: "license_activation",
        label: "訂閱激活",
        icon: <KeyRound className="size-4 shrink-0" aria-hidden />,
        onClick: () => setActivationOpen(true)
      },
      {
        key: "license_machine_code",
        label: "機器碼",
        icon: <Copy className="size-4 shrink-0" aria-hidden />,
        onClick: () => setMachineCodeOpen(true)
      },
      { type: "divider" as const },
      ...MORE_MENU_ITEMS.map(({ id, label, Icon }) => ({
        key: id,
        label: label,
        icon: <Icon className="size-4 shrink-0" aria-hidden />,
        disabled: true
      }))
    ],
    [supportLanguages, currentLanguage, switchLanguage]
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
              aria-label={"選單"}
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
            aria-label={"最小化"}
            onClick={() => void mainWindow.minimize()}
          >
            <Minus className="size-4" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className={controlBtnClass}
            aria-label={"最大化"}
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
            aria-label={"關閉"}
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
