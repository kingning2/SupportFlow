"use client";

import { FullModalWindowLayout } from "@/features/full/modal-window/modal-window-layout";

export default function ModalWindowLayoutRoute({ children }: { children: React.ReactNode }) {
  return <FullModalWindowLayout>{children}</FullModalWindowLayout>;
}
