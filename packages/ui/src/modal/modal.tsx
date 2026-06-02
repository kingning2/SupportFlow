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
  type ComponentType,
  type MouseEvent,
  type ReactNode
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";

import { getModalMotionTarget, playModalEnter, playModalExit } from "./motion/play-window-motion";
import {
  closeModalWindow,
  notifyModalWindowReady
} from "@supportflow/shared/tauri-bridge/cmd/window";
import { Button } from "@supportflow/ui/button";
import { TauriEvent, isModalPanel } from "@supportflow/shared/tauri-bridge/enums";
import { cn } from "@supportflow/shared";
import type { ModalOpenPanelPayload } from "@supportflow/shared/contracts/tauri-payloads";

import { useAppSelector } from "@supportflow/shared/desktop-shell/store/hooks";

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
  toolbar?: ReactNode;
  extra?: ReactNode;
  header?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
  className?: string;
  headerClassName?: string;
  bodyClassName?: string;
};

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

export type ModalPanelRegistry = Record<string, ComponentType>;

/** 监听 Rust `modal/open-panel`，渲染应用提供的 panels 注册表 */
export function ModalPanelHost({ registry }: { registry: ModalPanelRegistry }) {
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

  const Panel = registry[panelName];
  if (!Panel) {
    return (
      <Modal title={t("title")}>
        <p className="text-muted-foreground text-sm">
          {t("unknown_panel", { name: panelName || "—" })}
        </p>
      </Modal>
    );
  }

  return createElement(Panel);
}

export type ModalOverlayProps = {
  className?: string;
};

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
