"use client";

import DesktopRoot from "@supportflow/shared/desktop-shell/providers/desktop-root";

export default function GlobalProvider({ children }: { children: React.ReactNode }) {
  return <DesktopRoot>{children}</DesktopRoot>;
}
