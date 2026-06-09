"use client";

import { Layout } from "@/features/full/modal-window/layout";

export default function ModalWindowLayoutRoute({ children }: { children: React.ReactNode }) {
  return <Layout>{children}</Layout>;
}
