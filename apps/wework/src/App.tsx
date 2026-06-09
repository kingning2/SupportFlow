import { Route, Routes } from "react-router-dom";

import { DesktopAppLayout } from "@supportflow/ui/app-shell";

import { weworkShellAccent } from "@/config/wework-shell-accent";
import { WEWORK_SHELL_CONTENT_CLASS } from "@/config/wework-shell";

import { WeworkAppPage } from "@/features/wework/page";

export function App() {
  return (
    <DesktopAppLayout accent={weworkShellAccent} contentClassName={WEWORK_SHELL_CONTENT_CLASS}>
      <Routes>
        <Route path="*" element={<WeworkAppPage />} />
      </Routes>
    </DesktopAppLayout>
  );
}
