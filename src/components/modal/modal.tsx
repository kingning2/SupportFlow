"use client";

import { X } from "lucide-react";
import {
  createContext,
  createElement,
  useCallback,
  useContext,
  useLayoutEffect,
  useRef,
  useState,
  type MouseEvent,
  type ReactNode
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";

import {
  getModalMotionTarget,
  playModalEnter,
  playModalExit
} from "@/animation/modal/play-window-motion";
import { closeModalWindow, notifyModalWindowReady } from "@/cmd/window";
import { Button } from "@/components/ui/button";
import { TauriEvent, isModalPanel } from "@/enums";
import { cn } from "@/lib/utils";
import type { ModalOpenPanelPayload } from "@/types/tauri-payloads";

import { useAppSelector } from "@/store/hooks";

import { MODAL_PANEL_REGISTRY } from "./panels";

type ModalMotionContextValue = {
  requestClose: () => void;
  notifyPanelOpen: (panelName: string, openNonce: number) => void;
};

const ModalMotionContext = createContext<ModalMotionContextValue | null>(null);

function closeModalWindowSafe() {
  const label = getCurrentWebviewWindow().label;
  void closeModalWindow(label).catch(() => {
    void getCurrentWebviewWindow()
      .close()
      .catch(() => undefined);
  });
}

export function ModalMotionProvider({
  children,
  className
}: {
  children: ReactNode;
  className?: string;
}) {
  const closingRef = useRef(false);
  const readyGenerationRef = useRef(0);

  const notifyPanelOpen = useCallback((_panelName: string, openNonce: number) => {
    if (openNonce < 1) return;

    const generation = openNonce;
    readyGenerationRef.current = generation;
    closingRef.current = false;

    const label = getCurrentWebviewWindow().label;

    void notifyModalWindowReady(label)
      .then(async () => {
        if (readyGenerationRef.current !== generation) return;

        const target = getModalMotionTarget();
        if (!target) return;

        target.style.pointerEvents = "";
        await playModalEnter(target);
      })
      .catch(() => undefined);
  }, []);

  const requestClose = useCallback(() => {
    if (closingRef.current) return;
    closingRef.current = true;

    const target = getModalMotionTarget();
    if (!target) {
      closeModalWindowSafe();
      return;
    }

    target.style.pointerEvents = "none";
    playModalExit(target, closeModalWindowSafe);
  }, []);

  return (
    <ModalMotionContext.Provider value={{ requestClose, notifyPanelOpen }}>
      <div
        data-modal-motion-root
        className={cn(
          "modal-panel-root modal-window flex min-h-0 flex-1 flex-col overflow-hidden",
          className
        )}
      >
        {children}
      </div>
    </ModalMotionContext.Provider>
  );
}

export function useModalMotion() {
  const ctx = useContext(ModalMotionContext);
  return {
    requestClose: ctx?.requestClose ?? closeModalWindowSafe,
    notifyPanelOpen: ctx?.notifyPanelOpen ?? (() => undefined)
  };
}

export type ModalProps = {
  title?: ReactNode;
  /** 顶栏右上：刷新等图标，与关闭按钮同一行 */
  toolbar?: ReactNode;
  /** 顶栏右上：主操作按钮（在 toolbar 行下方，如輸入授權碼） */
  extra?: ReactNode;
  /** 自定义整块标题区（替换默认标题栏，面板自行处理拖动与关闭） */
  header?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
  className?: string;
  headerClassName?: string;
  bodyClassName?: string;
};

/** 子窗口标题栏拖动：与主窗一致，仅 `data-drag-region` 区域触发 */
export function onModalDragMouseDown(e: MouseEvent) {
  const isDragRegion = Boolean((e.target as HTMLElement).dataset.dragRegion);
  if (isDragRegion && e.buttons === 1) {
    void getCurrentWindow().startDragging();
  }
}

function DefaultModalHeader({
  title,
  toolbar,
  extra,
  headerClassName,
  onClose
}: {
  title?: ReactNode;
  toolbar?: ReactNode;
  extra?: ReactNode;
  headerClassName?: string;
  onClose: () => void;
}) {
  return (
    <header
      className={cn(
        "border-border/80 relative flex shrink-0 items-start gap-3 overflow-hidden border-b bg-white px-5 py-4 select-none",
        headerClassName
      )}
      onMouseDown={onModalDragMouseDown}
    >
      <div data-drag-region className="flex min-w-0 flex-1 items-start">
        <div className="pointer-events-none w-full min-w-0">{title}</div>
      </div>
      <div className="pointer-events-auto flex shrink-0 flex-col items-end gap-2">
        <div className="flex items-center gap-0.5">
          {toolbar}
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="size-8 shrink-0 text-slate-500 hover:bg-white/60 hover:text-slate-700"
            onClick={onClose}
            aria-label="Close"
          >
            <X className="size-4" />
          </Button>
        </div>
        {extra}
      </div>
    </header>
  );
}

/** Modal 子窗口基础壳：可拖动标题栏 + 关闭 + 内容区（对齐 antd Modal 结构） */
export function Modal({
  title,
  toolbar,
  extra,
  header,
  footer,
  children,
  className,
  headerClassName,
  bodyClassName
}: ModalProps) {
  const { requestClose } = useModalMotion();

  return (
    <div
      className={cn(
        "modal-window flex min-h-0 flex-1 flex-col overflow-hidden bg-white",
        className
      )}
    >
      {header ?? (
        <DefaultModalHeader
          title={title}
          toolbar={toolbar}
          extra={extra}
          headerClassName={headerClassName}
          onClose={() => requestClose()}
        />
      )}
      <div className={cn("min-h-0 flex-1 overflow-auto p-4", bodyClassName)}>{children}</div>
      {footer ? (
        <footer className="border-border/80 shrink-0 border-t bg-white px-4 py-3">{footer}</footer>
      ) : null}
    </div>
  );
}

/** 监听 Rust `modal/open-panel`，渲染 panels 注册表中的弹窗 */
export function ModalPanelHost() {
  const { t } = useTranslation("modal_window");
  const { notifyPanelOpen } = useModalMotion();
  const [panelName, setPanelName] = useState("");
  const [openNonce, setOpenNonce] = useState(0);

  const applyPanel = useCallback((name: string) => {
    setPanelName(name);
    setOpenNonce((n) => n + 1);
  }, []);

  useLayoutEffect(() => {
    const webview = getCurrentWebviewWindow();
    let unlisten: (() => void) | undefined;

    void webview
      .listen<ModalOpenPanelPayload>(TauriEvent.ModalOpenPanel, (event) => {
        applyPanel(event.payload.name);
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => {
      unlisten?.();
    };
  }, [applyPanel]);

  useLayoutEffect(() => {
    if (!panelName || openNonce < 1) return;
    notifyPanelOpen(panelName, openNonce);
  }, [notifyPanelOpen, openNonce, panelName]);

  if (!isModalPanel(panelName)) {
    return (
      <Modal title={t("title")}>
        <p className="text-muted-foreground text-sm">
          {t("unknown_panel", { name: panelName || "—" })}
        </p>
      </Modal>
    );
  }

  return createElement(MODAL_PANEL_REGISTRY[panelName]);
}

export type ModalOverlayProps = {
  className?: string;
};

/** 主窗蒙层：有 modal 子窗口时阻止主窗交互 */
export function ModalOverlay({ className }: ModalOverlayProps) {
  const openLabels = useAppSelector((state) => state.modal.openLabels);

  if (openLabels.length <= 0) return null;

  return (
    <div
      role="presentation"
      aria-hidden
      className={cn(
        "modal-overlay-enter pointer-events-auto absolute inset-0 z-200 bg-slate-900/45 backdrop-blur-[1px]",
        className
      )}
    />
  );
}
