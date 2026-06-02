"use client";

import { Suspense } from "react";

import { ModalPanelHost } from "@supportflow/ui/modal";
import { MODAL_PANEL_REGISTRY } from "@/components/modal/panels";

export default function ModalWindowPage() {
  return (
    <Suspense fallback={null}>
      <ModalPanelHost registry={MODAL_PANEL_REGISTRY} />
    </Suspense>
  );
}
