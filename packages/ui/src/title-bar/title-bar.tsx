"use client";

import {
  ArrowUpCircle,
  Check,
  CircleHelp,
  Globe,
  Headphones,
  Info,
  Mail,
  Menu,
  Minus,
  Square,
  X,
  type LucideIcon
} from "lucide-react";
import { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { setLang } from "@supportflow/shared/tauri-bridge/cmd/lang";
import { mainWindow } from "@supportflow/shared/tauri-bridge/window/main-window";
import { Button } from "@supportflow/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger
} from "@supportflow/ui/dropdown-menu";
import { cn } from "@supportflow/shared";
import { useAppDispatch, useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";
import { changeCurrentLanguageAction } from "@supportflow/shared/desktop-shell/store/modules/app";
import type { Language } from "@supportflow/shared/tauri-bridge/enums";

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
  "button, a, input, select, textarea, [role='menuitem'], [data-radix-popper-content-wrapper]";

type MoreMenuItem = {
  id: string;
  i18nKey:
    | "menu_feedback"
    | "menu_contact_support"
    | "menu_online_help"
    | "menu_check_updates"
    | "menu_about";
  Icon: LucideIcon;
};

const MORE_MENU_ITEMS: MoreMenuItem[] = [
  { id: "feedback", i18nKey: "menu_feedback", Icon: Mail },
  { id: "contact_support", i18nKey: "menu_contact_support", Icon: Headphones },
  { id: "online_help", i18nKey: "menu_online_help", Icon: CircleHelp },
  { id: "check_updates", i18nKey: "menu_check_updates", Icon: ArrowUpCircle },
  { id: "about", i18nKey: "menu_about", Icon: Info }
];

/** 整栏可拖；仅排除按钮/菜单等可交互控件（由控件区 stopPropagation 兜底） */
function handleTitleBarMouseDown(e: React.MouseEvent<HTMLDivElement>) {
  if (e.buttons !== 1) return;
  const target = e.target as HTMLElement;
  if (target.closest(INTERACTIVE_TITLE_BAR_SELECTOR)) return;
  void mainWindow.startDragging();
}

const TitleBar = memo((props: { height?: number; accent?: TitleBarAccent }) => {
  const { t } = useTranslation("title_bar");
  const h = props.height ?? 40;
  const accent = props.accent;
  const title = accent?.title ?? t("app_name");
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
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className={controlBtnClass}
                aria-label={t("menu")}
              >
                <Menu className="size-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              sideOffset={6}
              className="min-w-[200px]"
              onPointerDown={(e) => e.stopPropagation()}
            >
              <DropdownMenuSub>
                <DropdownMenuSubTrigger className="gap-2">
                  <Globe className="size-4 shrink-0 opacity-80" aria-hidden />
                  <span>{t("menu_language")}</span>
                </DropdownMenuSubTrigger>
                <DropdownMenuSubContent className="min-w-40" sideOffset={4}>
                  {supportLanguages.map((opt) => (
                    <DropdownMenuItem
                      key={opt.value}
                      className="gap-2 pl-2"
                      onSelect={() => {
                        void switchLanguage(opt.value);
                      }}
                    >
                      <span className="flex size-4 shrink-0 items-center justify-center">
                        {currentLanguage === opt.value ? (
                          <Check className="text-primary size-4" aria-hidden />
                        ) : null}
                      </span>
                      {opt.label}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuSubContent>
              </DropdownMenuSub>

              <DropdownMenuSeparator />

              {MORE_MENU_ITEMS.map(({ id, i18nKey, Icon }) => (
                <DropdownMenuItem key={id} disabled className="gap-2">
                  <Icon className="size-4 shrink-0" aria-hidden />
                  {t(i18nKey)}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          <Button
            type="button"
            variant="ghost"
            size="icon"
            className={controlBtnClass}
            aria-label={t("minimize")}
            onClick={() => void mainWindow.minimize()}
          >
            <Minus className="size-4" />
          </Button>
          {false && (
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className={controlBtnClass}
              aria-label={t("maximize")}
              onClick={() => void mainWindow.toggleMaximize()}
            >
              <Square className="size-3.5" />
            </Button>
          )}
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
            aria-label={t("close")}
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
