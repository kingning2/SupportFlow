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

import { setLang } from "@/cmd/lang";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger
} from "@/components/ui/dropdown-menu";
import { mainWindow } from "@/config/windows";
import { cn } from "@/lib/utils";
import { useAppDispatch, useAppSelector } from "@/store/hooks";
import { changeCurrentLanguageAction } from "@/store/modules/app";
import type { Language } from "@/enums";

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

const TitleBar = memo((props: { height?: number }) => {
  const { t } = useTranslation("title_bar");
  const h = props.height ?? 40;
  const dispatch = useAppDispatch();
  const currentLanguage = useAppSelector((state) => state.app.currentLanguage);
  const supportLanguages = useAppSelector((state) => state.app.supportLanguages);

  const switchLanguage = useCallback(
    async (next: Language) => {
      if (next === currentLanguage) return;
      try {
        await setLang(next);
        /* 语言写入 Rust 后会广播 session/changed，各 Webview Redux 由 cross-webview-sync 订阅更新 */
      } catch {
        dispatch(changeCurrentLanguageAction(next));
      }
    },
    [currentLanguage, dispatch]
  );

  /** 与 unlock 一致：仅当事件目标带 `data-drag-region` 时才拖动（按钮内部无此属性） */
  function handleBarMouseDown(e: React.MouseEvent) {
    const isDragRegion = Boolean((e.target as HTMLElement).dataset.dragRegion);
    if (isDragRegion && e.buttons === 1) {
      void mainWindow.startDragging();
    }
  }

  return (
    <div
      role="banner"
      data-drag-region
      className={cn(
        "bg-card/90 flex w-full items-center justify-between px-3 backdrop-blur select-none"
      )}
      style={{ height: h }}
      onMouseDown={handleBarMouseDown}
    >
      <div className="pointer-events-none flex min-w-0 flex-1 items-center gap-2">
        <div
          className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-linear-to-br from-[#2b7fff] to-[#155dfc] text-sm font-bold text-white"
          aria-hidden
        >
          T
        </div>
        <span className="text-foreground truncate text-[15px] font-semibold tracking-tight">
          {t("app_name")}
        </span>
      </div>

      <div className="pointer-events-auto flex shrink-0 items-center gap-2" data-drag-region>
        <div className="ml-1 flex items-center gap-0.5">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="text-muted-foreground"
                aria-label={t("menu")}
                onPointerDown={(e) => e.stopPropagation()}
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
            className="text-muted-foreground"
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
              className="text-muted-foreground"
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
            className="text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
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
