"use client";

import { Suspense } from "react";

import { ModalPanelHost } from "@supportflow/ui/modal";

import { MODAL_PANEL_REGISTRY } from "./panels";

export function Page() {
  return (
    <Suspense fallback={null}>
      <ModalPanelHost registry={MODAL_PANEL_REGISTRY} />
    </Suspense>
  );
}
