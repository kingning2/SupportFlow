"use client";

import { Suspense } from "react";

import { ModalPanelHost } from "@/components/modal";

export default function ModalWindowPage() {
  return (
    <Suspense fallback={null}>
      <ModalPanelHost />
    </Suspense>
  );
}
