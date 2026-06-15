"use client";

import { IconClose } from "@douyinfe/semi-icons";
import { IconButton } from "@douyinfe/semi-ui-19";
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

import { getModalMotionTarget, playModalEnter, playModalExit } from "./motion/play-window-motion";
import {
  closeModalWindow,
  notifyModalWindowReady
} from "@supportflow/shared/tauri-bridge/cmd/window";
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
      style={{
        position: "relative",
        display: "flex",
        flexShrink: 0,
        alignItems: "flex-start",
        gap: 12,
        overflow: "hidden",
        borderBottom: "1px solid var(--semi-color-border)",
        background: "var(--semi-color-bg-0)",
        padding: "16px 20px",
        userSelect: "none"
      }}
      className={headerClassName}
      onMouseDown={onModalDragMouseDown}
    >
      <div
        data-drag-region
        style={{ display: "flex", minWidth: 0, flex: 1, alignItems: "flex-start" }}
      >
        <div style={{ pointerEvents: "none", width: "100%", minWidth: 0 }}>{title}</div>
      </div>
      <div
        style={{
          pointerEvents: "auto",
          display: "flex",
          flexShrink: 0,
          flexDirection: "column",
          alignItems: "flex-end",
          gap: 8
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 2 }}>
          {toolbar}
          <IconButton
            icon={<IconClose />}
            aria-label="Close"
            theme="borderless"
            type="tertiary"
            onClick={onClose}
          />
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
      style={{
        display: "flex",
        minHeight: 0,
        flex: 1,
        flexDirection: "column",
        overflow: "hidden",
        background: "var(--semi-color-bg-0)"
      }}
      className={className}
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
      <div
        style={{ minHeight: 0, flex: 1, overflow: "auto", padding: 16 }}
        className={bodyClassName}
      >
        {children}
      </div>
      {footer ? (
        <footer
          style={{
            flexShrink: 0,
            borderTop: "1px solid var(--semi-color-border)",
            background: "var(--semi-color-bg-0)",
            padding: "12px 16px"
          }}
        >
          {footer}
        </footer>
      ) : null}
    </div>
  );
}

export type ModalPanelRegistry = Record<string, ComponentType>;

/** 监听 Rust `modal/open-panel`，渲染应用提供的 panels 注册表 */
export function ModalPanelHost({ registry }: { registry: ModalPanelRegistry }) {
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
      <Modal title={"視窗"}>
        <p
          style={{ fontSize: 14, color: "var(--semi-color-text-2)" }}
        >{`未知面板：${panelName || "—"}`}</p>
      </Modal>
    );
  }

  const Panel = registry[panelName];
  if (!Panel) {
    return (
      <Modal title={"視窗"}>
        <p
          style={{ fontSize: 14, color: "var(--semi-color-text-2)" }}
        >{`未知面板：${panelName || "—"}`}</p>
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
      style={{
        position: "absolute",
        inset: 0,
        zIndex: 200,
        pointerEvents: "auto",
        background: "rgb(15 23 42 / 0.45)",
        backdropFilter: "blur(1px)"
      }}
      className={className}
    />
  );
}
